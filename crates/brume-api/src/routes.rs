//! The routes of `/api`: each one turns the request into an `ApiCall`, sends it to the engine
//! with the session cookie, and turns the answer into the status and JSON body of the HTTP
//! contract.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use brume_proto::api::{
    ApiCall, ApiFailure, ApiValue, FailureKind, LogStream, NewChangeDto, NewProjectSourceDto,
    NewSharedVariableDto, VariableLevelDto,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::engine_client::EngineError;
use crate::{AppState, csrf, session, sse};

/// The default page size of `GET /api/events`.
const DEFAULT_EVENTS_LIMIT: u32 = 50;

/// The largest page size of `GET /api/events`.
const MAX_EVENTS_LIMIT: u32 = 200;

/// The routes under `/api`.
pub(crate) fn api() -> Router<Arc<AppState>> {
    Router::new()
        .route("/session", post(login).get(current_session).delete(logout))
        .route("/projects", get(list_projects).post(create_project))
        .route(
            "/projects/{project_id}/environments",
            post(create_environment),
        )
        .route("/environments", get(list_environments))
        .route(
            "/environments/{environment_id}/services",
            post(create_service),
        )
        .route(
            "/environments/{environment_id}/changes/deploy",
            post(deploy_environment_changes),
        )
        .route(
            "/environments/{environment_id}/changes",
            axum::routing::delete(discard_environment_changes),
        )
        .route(
            "/environments/{environment_id}/logs",
            get(sse::environment_logs),
        )
        .route("/services", get(list_services))
        .route(
            "/services/{service_id}",
            axum::routing::delete(delete_service),
        )
        .route("/services/{service_id}/settings", get(service_settings))
        .route("/services/{service_id}/redeploy", post(redeploy))
        .route("/deployments", get(list_deployments))
        .route("/deployments/{deployment_id}/rollback", post(rollback))
        .route("/deployments/{deployment_id}/revision", get(revision))
        .route("/deployments/{deployment_id}/logs", get(deployment_logs))
        .route("/variables", get(list_variables).post(set_shared_variable))
        .route("/variables/impact", get(variable_impact))
        .route(
            "/variables/{variable_id}",
            axum::routing::delete(remove_shared_variable),
        )
        .route("/variables/{variable_id}/reveal", post(reveal_variable))
        .route("/changes", get(list_changes).post(stage_change))
        .route(
            "/changes/{change_id}",
            axum::routing::delete(discard_change),
        )
        .route("/events", get(list_events))
        .route("/github", any(github_unavailable))
        .route("/github/{*rest}", any(github_unavailable))
        .fallback(not_found)
}

/// `{"message": …}` with `status`.
pub(crate) fn json_error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}

/// The status of a failed call, with the engine's message.
pub(crate) fn failure_response(failure: ApiFailure) -> Response {
    let status = match failure.kind {
        FailureKind::Unauthorized => StatusCode::UNAUTHORIZED,
        FailureKind::NotFound => StatusCode::NOT_FOUND,
        FailureKind::Invalid => StatusCode::BAD_REQUEST,
        FailureKind::Conflict => StatusCode::CONFLICT,
        FailureKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    json_error(status, &failure.message)
}

/// The answer when the engine does not answer.
pub(crate) fn engine_error_response(error: EngineError) -> Response {
    tracing::warn!(%error, "engine call failed");
    match error {
        EngineError::Unreachable => json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "The engine is not reachable.",
        ),
        EngineError::Timeout => json_error(
            StatusCode::GATEWAY_TIMEOUT,
            "The engine did not answer in time.",
        ),
        EngineError::Protocol => json_error(
            StatusCode::BAD_GATEWAY,
            "The engine sent an unexpected answer.",
        ),
    }
}

/// A JSON body whose errors answer `{"message": …}` and never quote the body, which can carry
/// a secret.
pub(crate) struct JsonBody<T>(pub(crate) T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(request, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(body_rejection(&rejection)),
        }
    }
}

fn body_rejection(rejection: &JsonRejection) -> Response {
    match rejection.status() {
        StatusCode::PAYLOAD_TOO_LARGE => {
            json_error(StatusCode::PAYLOAD_TOO_LARGE, csrf::BODY_TOO_LARGE)
        }
        StatusCode::UNSUPPORTED_MEDIA_TYPE => json_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "The request body must be JSON.",
        ),
        _ => json_error(
            StatusCode::BAD_REQUEST,
            "The request body does not fit this route.",
        ),
    }
}

/// Query parameters whose errors answer 400 `{"message": …}`.
pub(crate) struct QueryParams<T>(pub(crate) T);

impl<S, T> FromRequestParts<S> for QueryParams<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(value)) => Ok(Self(value)),
            Err(_) => Err(json_error(
                StatusCode::BAD_REQUEST,
                "The query parameters do not fit this route.",
            )),
        }
    }
}

/// Sends `body` to the engine with the request's session, and answers `status` with the
/// value.
async fn forward(
    state: &AppState,
    headers: &HeaderMap,
    body: ApiCall,
    status: StatusCode,
) -> Response {
    match state.engine.call(session::token(headers), body).await {
        Ok(Ok(value)) => value_response(value, status),
        Ok(Err(failure)) => failure_response(failure),
        Err(error) => engine_error_response(error),
    }
}

/// The JSON body of a value, with `status`; `NoContent` is a 204 whatever the route.
fn value_response(value: ApiValue, status: StatusCode) -> Response {
    match value {
        ApiValue::NoContent => StatusCode::NO_CONTENT.into_response(),
        ApiValue::CurrentSession(session) => (status, Json(session)).into_response(),
        ApiValue::Projects(projects) => (status, Json(projects)).into_response(),
        ApiValue::CreatedProject(created) => (status, Json(created)).into_response(),
        ApiValue::Environments(environments) => (status, Json(environments)).into_response(),
        ApiValue::Environment(environment) => (status, Json(environment)).into_response(),
        ApiValue::Services(services) => (status, Json(services)).into_response(),
        ApiValue::Service(service) => (status, Json(service)).into_response(),
        ApiValue::ServiceSettings(settings) => (status, Json(settings)).into_response(),
        ApiValue::Deployments(deployments) => (status, Json(deployments)).into_response(),
        ApiValue::Deployment(deployment) => (status, Json(deployment)).into_response(),
        ApiValue::Revision(revision) => (status, Json(revision)).into_response(),
        ApiValue::LogLines(lines) => (status, Json(lines)).into_response(),
        ApiValue::Variables(variables) => (status, Json(variables)).into_response(),
        ApiValue::VariableImpact(impact) => (status, Json(impact)).into_response(),
        ApiValue::SharedVariableSet(set) => (status, Json(set)).into_response(),
        ApiValue::SharedVariableRemoved(removed) => (status, Json(removed)).into_response(),
        ApiValue::RevealedValue(revealed) => (status, Json(revealed)).into_response(),
        ApiValue::Changes(changes) => (status, Json(changes)).into_response(),
        ApiValue::Change(change) => (status, Json(change)).into_response(),
        ApiValue::ChangesDeployed(deployed) => (status, Json(deployed)).into_response(),
        ApiValue::Events(page) => (status, Json(page)).into_response(),
        // The token goes into the cookie, in `login`, and never into a body.
        ApiValue::Session { .. } | ApiValue::LogStreamOpened => {
            engine_error_response(EngineError::Protocol)
        }
    }
}

#[derive(Deserialize)]
struct LoginBody {
    token: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    JsonBody(body): JsonBody<LoginBody>,
) -> Response {
    let call = ApiCall::Login { token: body.token };
    match state.engine.call(None, call).await {
        Ok(Ok(ApiValue::Session { token, .. })) => match session::set_cookie(&token) {
            Some(cookie) => (StatusCode::NO_CONTENT, [cookie]).into_response(),
            None => engine_error_response(EngineError::Protocol),
        },
        Ok(Ok(_)) => engine_error_response(EngineError::Protocol),
        Ok(Err(failure)) => failure_response(failure),
        Err(error) => engine_error_response(error),
    }
}

async fn current_session(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::GetSession, StatusCode::OK).await
}

/// Clears the cookie whatever the engine answers: the browser forgets the session anyway.
async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let mut response = forward(&state, &headers, ApiCall::Logout, StatusCode::NO_CONTENT).await;
    let (name, value) = session::clear_cookie();
    response.headers_mut().insert(name, value);
    response
}

async fn list_projects(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::ListProjects, StatusCode::OK).await
}

#[derive(Deserialize)]
struct NewProjectBody {
    source: NewProjectSourceDto,
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    JsonBody(body): JsonBody<NewProjectBody>,
) -> Response {
    let unavailable = match &body.source {
        NewProjectSourceDto::Empty | NewProjectSourceDto::Image { .. } => None,
        NewProjectSourceDto::Github { .. } => Some("github"),
        NewProjectSourceDto::Static { .. } => Some("static"),
        NewProjectSourceDto::Postgres => Some("postgres"),
        NewProjectSourceDto::Mongodb => Some("mongodb"),
        NewProjectSourceDto::Redis => Some("redis"),
    };
    if let Some(kind) = unavailable {
        return json_error(
            StatusCode::BAD_REQUEST,
            &format!("{kind} sources are not available yet."),
        );
    }
    let call = ApiCall::CreateProject {
        source: body.source,
    };
    forward(&state, &headers, call, StatusCode::CREATED).await
}

#[derive(Deserialize)]
struct NewEnvironmentBody {
    name: String,
}

async fn create_environment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    JsonBody(body): JsonBody<NewEnvironmentBody>,
) -> Response {
    let call = ApiCall::CreateEnvironment {
        project_id,
        name: body.name,
    };
    forward(&state, &headers, call, StatusCode::CREATED).await
}

async fn list_environments(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::ListEnvironments, StatusCode::OK).await
}

#[derive(Deserialize)]
struct NewServiceBody {
    image: String,
    name: Option<String>,
}

async fn create_service(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(environment_id): Path<String>,
    JsonBody(body): JsonBody<NewServiceBody>,
) -> Response {
    let call = ApiCall::CreateService {
        environment_id,
        image: body.image,
        name: body.name,
    };
    forward(&state, &headers, call, StatusCode::CREATED).await
}

async fn deploy_environment_changes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(environment_id): Path<String>,
) -> Response {
    let call = ApiCall::DeployEnvironmentChanges { environment_id };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn discard_environment_changes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(environment_id): Path<String>,
) -> Response {
    let call = ApiCall::DiscardEnvironmentChanges { environment_id };
    forward(&state, &headers, call, StatusCode::NO_CONTENT).await
}

async fn list_services(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::ListServices, StatusCode::OK).await
}

async fn delete_service(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(service_id): Path<String>,
) -> Response {
    let call = ApiCall::DeleteService { service_id };
    forward(&state, &headers, call, StatusCode::NO_CONTENT).await
}

async fn service_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(service_id): Path<String>,
) -> Response {
    let call = ApiCall::GetServiceSettings { service_id };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn redeploy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(service_id): Path<String>,
) -> Response {
    let call = ApiCall::Redeploy { service_id };
    forward(&state, &headers, call, StatusCode::CREATED).await
}

async fn list_deployments(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::ListDeployments, StatusCode::OK).await
}

async fn rollback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(deployment_id): Path<String>,
) -> Response {
    let call = ApiCall::Rollback { deployment_id };
    forward(&state, &headers, call, StatusCode::CREATED).await
}

async fn revision(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(deployment_id): Path<String>,
) -> Response {
    let call = ApiCall::GetRevision { deployment_id };
    forward(&state, &headers, call, StatusCode::OK).await
}

#[derive(Deserialize)]
struct DeploymentLogsQuery {
    stream: LogStream,
}

async fn deployment_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(deployment_id): Path<String>,
    QueryParams(params): QueryParams<DeploymentLogsQuery>,
) -> Response {
    let call = ApiCall::DeploymentLogs {
        deployment_id,
        stream: params.stream,
    };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn list_variables(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::ListVariables, StatusCode::OK).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VariableImpactQuery {
    level: VariableLevelDto,
    owner_id: String,
    name: String,
}

async fn variable_impact(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    QueryParams(params): QueryParams<VariableImpactQuery>,
) -> Response {
    let call = ApiCall::VariableImpact {
        level: params.level,
        owner_id: params.owner_id,
        name: params.name,
    };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn set_shared_variable(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    JsonBody(body): JsonBody<NewSharedVariableDto>,
) -> Response {
    let call = ApiCall::SetSharedVariable(body);
    forward(&state, &headers, call, StatusCode::CREATED).await
}

async fn remove_shared_variable(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(variable_id): Path<String>,
) -> Response {
    let call = ApiCall::RemoveSharedVariable { variable_id };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn reveal_variable(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(variable_id): Path<String>,
) -> Response {
    let call = ApiCall::RevealVariable { variable_id };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn list_changes(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    forward(&state, &headers, ApiCall::ListChanges, StatusCode::OK).await
}

async fn stage_change(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    JsonBody(body): JsonBody<NewChangeDto>,
) -> Response {
    let call = ApiCall::StageChange(body);
    forward(&state, &headers, call, StatusCode::CREATED).await
}

async fn discard_change(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(change_id): Path<String>,
) -> Response {
    let call = ApiCall::DiscardChange { change_id };
    forward(&state, &headers, call, StatusCode::NO_CONTENT).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventsQuery {
    project_id: String,
    before: Option<u64>,
    limit: Option<u32>,
}

async fn list_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    QueryParams(params): QueryParams<EventsQuery>,
) -> Response {
    let call = ApiCall::ListEvents {
        project_id: params.project_id,
        before: params.before,
        limit: params
            .limit
            .unwrap_or(DEFAULT_EVENTS_LIMIT)
            .min(MAX_EVENTS_LIMIT),
    };
    forward(&state, &headers, call, StatusCode::OK).await
}

async fn github_unavailable() -> Response {
    json_error(
        StatusCode::NOT_FOUND,
        "GitHub sources are not available yet.",
    )
}

async fn not_found() -> Response {
    json_error(StatusCode::NOT_FOUND, "This route does not exist.")
}
