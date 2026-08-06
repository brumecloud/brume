use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use brume_core::{
    ReviewComment, ReviewCommentsResponse, ReviewRoundSummary, ReviewRoundsResponse, ReviewStatus,
    ReviewStatusResponse, ReviewThread,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{access, auth, auth::AuthUser, error::ApiError, state::AppState};

const MAX_REQUEST_BYTES: usize = 16 * 1024;
const MAX_BODY_CHARS: usize = 4_096;
const MAX_DISPLAY_NAME_CHARS: usize = 64;
const MAX_PAGE_PATH_CHARS: usize = 1_024;
const MAX_ANCHOR_BYTES: usize = 8 * 1024;
const MAX_ANCHOR_EXACT_CHARS: usize = 512;
const MAX_ANCHOR_CONTEXT_CHARS: usize = 64;
const MAX_ANCHOR_SELECTOR_CHARS: usize = 512;
const MAX_THREADS_PER_ROUND: i64 = 500;
const MAX_COMMENTS_PER_THREAD: i64 = 200;

pub fn site_router() -> Router<AppState> {
    Router::new()
        .route(
            "/_brume/review/threads",
            get(site_threads).post(site_create_thread),
        )
        .route(
            "/_brume/review/threads/{thread_id}/comments",
            post(site_reply),
        )
        .route("/_brume/review/finish", post(site_finish))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/plans/{selector}/review", get(plan_review_status))
        .route(
            "/api/v1/plans/{selector}/review/comments",
            get(plan_review_comments),
        )
        .route(
            "/api/v1/plans/{selector}/review/rounds",
            get(plan_review_rounds),
        )
        .route(
            "/api/v1/deployments/{selector}/review",
            get(deployment_review_status),
        )
        .route(
            "/api/v1/deployments/{selector}/review/comments",
            get(deployment_review_comments),
        )
        .route(
            "/api/v1/deployments/{selector}/review/rounds",
            get(deployment_review_rounds),
        )
}

#[derive(Serialize)]
pub struct ReviewOverlayState {
    pub id: Uuid,
    pub number: i32,
    pub status: ReviewStatus,
    pub thread_count: i64,
    pub comment_count: i64,
}

pub async fn overlay_review_state(
    state: &AppState,
    control_id: Uuid,
) -> Result<Option<ReviewOverlayState>, ApiError> {
    Ok(visible_round(state, control_id)
        .await?
        .map(|round| ReviewOverlayState {
            id: round.id,
            number: round.number,
            status: round.status,
            thread_count: round.thread_count,
            comment_count: round.comment_count,
        }))
}

pub async fn supersede_open_round(
    transaction: &mut Transaction<'_, Postgres>,
    control_id: Uuid,
) -> Result<(), ApiError> {
    sqlx::query(
        "UPDATE review_rounds SET status = 'superseded', finished_at = now()
         WHERE access_control_id = $1 AND status = 'open'",
    )
    .bind(control_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn start_round(
    transaction: &mut Transaction<'_, Postgres>,
    control_id: Uuid,
) -> Result<Uuid, ApiError> {
    let round_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO review_rounds (id, access_control_id, round_number)
         VALUES (
            $1, $2,
            (SELECT COALESCE(MAX(round_number), 0) + 1
             FROM review_rounds WHERE access_control_id = $2)
         )",
    )
    .bind(round_id)
    .bind(control_id)
    .execute(&mut **transaction)
    .await?;
    Ok(round_id)
}

const ROUND_QUERY: &str = "SELECT review_rounds.id, review_rounds.round_number,
        review_rounds.status, review_rounds.created_at, review_rounds.finished_at,
        (SELECT count(*) FROM review_threads
         WHERE review_threads.round_id = review_rounds.id) AS thread_count,
        (SELECT count(*) FROM review_comments
         JOIN review_threads ON review_threads.id = review_comments.thread_id
         WHERE review_threads.round_id = review_rounds.id) AS comment_count
 FROM review_rounds";

fn round_from_row(row: &sqlx::postgres::PgRow) -> Result<ReviewRoundSummary, ApiError> {
    Ok(ReviewRoundSummary {
        id: row.try_get("id")?,
        number: row.try_get("round_number")?,
        status: row
            .try_get::<String, _>("status")?
            .parse()
            .map_err(ApiError::internal)?,
        created_at: row.try_get("created_at")?,
        finished_at: row.try_get("finished_at")?,
        thread_count: row.try_get("thread_count")?,
        comment_count: row.try_get("comment_count")?,
    })
}

pub async fn latest_round(
    state: &AppState,
    control_id: Uuid,
) -> Result<Option<ReviewRoundSummary>, ApiError> {
    sqlx::query(&format!(
        "{ROUND_QUERY} WHERE access_control_id = $1 ORDER BY round_number DESC LIMIT 1"
    ))
    .bind(control_id)
    .fetch_optional(&state.database)
    .await?
    .as_ref()
    .map(round_from_row)
    .transpose()
}

async fn visible_round(
    state: &AppState,
    control_id: Uuid,
) -> Result<Option<ReviewRoundSummary>, ApiError> {
    sqlx::query(&format!(
        "{ROUND_QUERY} WHERE access_control_id = $1 AND status IN ('open', 'finished')
         ORDER BY round_number DESC LIMIT 1"
    ))
    .bind(control_id)
    .fetch_optional(&state.database)
    .await?
    .as_ref()
    .map(round_from_row)
    .transpose()
}

async fn round_by_number(
    state: &AppState,
    control_id: Uuid,
    number: i32,
) -> Result<Option<ReviewRoundSummary>, ApiError> {
    sqlx::query(&format!(
        "{ROUND_QUERY} WHERE access_control_id = $1 AND round_number = $2"
    ))
    .bind(control_id)
    .bind(number)
    .fetch_optional(&state.database)
    .await?
    .as_ref()
    .map(round_from_row)
    .transpose()
}

async fn round_threads(state: &AppState, round_id: Uuid) -> Result<Vec<ReviewThread>, ApiError> {
    let thread_rows = sqlx::query(
        "SELECT id, page_path, anchor, created_at
         FROM review_threads WHERE round_id = $1 ORDER BY created_at",
    )
    .bind(round_id)
    .fetch_all(&state.database)
    .await?;
    let mut threads = thread_rows
        .iter()
        .map(|row| {
            Ok::<_, ApiError>(ReviewThread {
                id: row.try_get("id")?,
                page_path: row.try_get("page_path")?,
                anchor: row.try_get("anchor")?,
                created_at: row.try_get("created_at")?,
                comments: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let comment_rows = sqlx::query(
        "SELECT review_comments.id, review_comments.thread_id, review_comments.author_public_id,
                review_comments.author_display_name, review_comments.author_is_owner,
                review_comments.body, review_comments.created_at
         FROM review_comments
         JOIN review_threads ON review_threads.id = review_comments.thread_id
         WHERE review_threads.round_id = $1
         ORDER BY review_comments.created_at",
    )
    .bind(round_id)
    .fetch_all(&state.database)
    .await?;
    for row in &comment_rows {
        let thread_id: Uuid = row.try_get("thread_id")?;
        let comment = ReviewComment {
            id: row.try_get("id")?,
            author_public_id: row.try_get("author_public_id")?,
            author_display_name: row.try_get("author_display_name")?,
            author_is_owner: row.try_get("author_is_owner")?,
            body: row.try_get("body")?,
            created_at: row.try_get("created_at")?,
        };
        if let Some(thread) = threads.iter_mut().find(|thread| thread.id == thread_id) {
            thread.comments.push(comment);
        }
    }
    Ok(threads)
}

async fn comments_response(
    state: &AppState,
    round: ReviewRoundSummary,
) -> Result<ReviewCommentsResponse, ApiError> {
    let threads = round_threads(state, round.id).await?;
    Ok(ReviewCommentsResponse { round, threads })
}

fn validate_body_text(body: &str) -> Result<String, ApiError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("Comment text cannot be empty"));
    }
    if trimmed.chars().count() > MAX_BODY_CHARS {
        return Err(ApiError::bad_request(format!(
            "Comment text exceeds {MAX_BODY_CHARS} characters"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_display_name(display_name: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(name) = display_name.map(str::trim).filter(|name| !name.is_empty()) else {
        return Ok(None);
    };
    if name.chars().count() > MAX_DISPLAY_NAME_CHARS {
        return Err(ApiError::bad_request(format!(
            "Reviewer name exceeds {MAX_DISPLAY_NAME_CHARS} characters"
        )));
    }
    Ok(Some(name.to_owned()))
}

fn validate_page_path(page_path: &str) -> Result<(), ApiError> {
    if !page_path.starts_with('/') || page_path.chars().count() > MAX_PAGE_PATH_CHARS {
        return Err(ApiError::bad_request("Invalid comment page path"));
    }
    Ok(())
}

fn string_field<'a>(
    anchor: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
    required: bool,
    max_chars: usize,
) -> Result<Option<&'a str>, ApiError> {
    match anchor.get(field) {
        Some(serde_json::Value::String(value)) => {
            if value.is_empty() || value.chars().count() > max_chars {
                return Err(ApiError::bad_request(format!(
                    "Comment anchor field `{field}` must contain between 1 and {max_chars} characters"
                )));
            }
            Ok(Some(value))
        }
        None if !required => Ok(None),
        _ => Err(ApiError::bad_request(format!(
            "Comment anchor field `{field}` is missing or invalid"
        ))),
    }
}

fn validate_anchor(anchor: &serde_json::Value) -> Result<(), ApiError> {
    let serialized = serde_json::to_vec(anchor).map_err(ApiError::bad_request)?;
    if serialized.len() > MAX_ANCHOR_BYTES {
        return Err(ApiError::bad_request("Comment anchor is too large"));
    }
    let object = anchor
        .as_object()
        .ok_or_else(|| ApiError::bad_request("Comment anchor must be an object"))?;
    match object.get("type").and_then(serde_json::Value::as_str) {
        Some("text") => {
            string_field(object, "exact", true, MAX_ANCHOR_EXACT_CHARS)?;
            string_field(object, "prefix", false, MAX_ANCHOR_CONTEXT_CHARS)?;
            string_field(object, "suffix", false, MAX_ANCHOR_CONTEXT_CHARS)?;
        }
        Some("element") => {
            string_field(object, "css", true, MAX_ANCHOR_SELECTOR_CHARS)?;
            string_field(object, "tag", true, 32)?;
            string_field(object, "src", false, 2_048)?;
            string_field(object, "text_digest", false, MAX_ANCHOR_CONTEXT_CHARS)?;
        }
        _ => {
            return Err(ApiError::bad_request(
                "Comment anchor type must be `text` or `element`",
            ));
        }
    }
    Ok(())
}

struct SiteViewer {
    control: access::AccessControl,
    owner: bool,
    identity: Option<access::Identity>,
}

async fn authorize_site_viewer(
    state: &AppState,
    cookies: &Cookies,
    headers: &HeaderMap,
    site: Uuid,
    return_to: &str,
) -> Result<SiteViewer, ApiError> {
    let control = access::load_control(state, site).await?;
    let location = access::validate_return_to(state, control.id, return_to).await?;
    if let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        && origin != location.origin
    {
        return Err(ApiError::forbidden("Invalid review request origin"));
    }
    let owner = access::site_session_is_owner(state, cookies, &control).await?
        || auth::web_user(state, cookies)
            .await?
            .is_some_and(|user| user.id == control.owner_user_id);
    if !matches!(
        access::authorize_request(state, cookies, &control, return_to, owner, None).await?,
        access::RequestAuthorization::Allowed
    ) {
        return Err(ApiError::not_found());
    }
    let identity = access::identity_from_cookie(state, cookies).await?;
    Ok(SiteViewer {
        control,
        owner,
        identity,
    })
}

fn author_binds(
    viewer: &SiteViewer,
    display_name: Option<String>,
) -> (Option<Uuid>, Option<String>, Option<String>, bool) {
    let identity_id = viewer.identity.as_ref().map(|identity| identity.id);
    let public_id = viewer
        .identity
        .as_ref()
        .map(|identity| identity.public_id.clone());
    let display_name = if viewer.owner || viewer.identity.is_some() {
        None
    } else {
        display_name
    };
    (identity_id, public_id, display_name, viewer.owner)
}

#[derive(Deserialize)]
struct SiteQuery {
    site: Uuid,
    return_to: String,
}

async fn site_threads(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Query(query): Query<SiteQuery>,
) -> Result<Response, ApiError> {
    let viewer = authorize_site_viewer(&state, &cookies, &headers, query.site, &query.return_to)
        .await?;
    let response = match visible_round(&state, viewer.control.id).await? {
        Some(round) => {
            let threads = round_threads(&state, round.id).await?;
            SiteThreadsResponse {
                round: Some(round),
                threads,
            }
        }
        None => SiteThreadsResponse {
            round: None,
            threads: Vec::new(),
        },
    };
    Ok(no_store(Json(response).into_response()))
}

#[derive(Serialize)]
struct SiteThreadsResponse {
    round: Option<ReviewRoundSummary>,
    threads: Vec<ReviewThread>,
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, no-store"),
    );
    response
}

#[derive(Deserialize)]
struct CreateThreadRequest {
    page_path: String,
    anchor: serde_json::Value,
    body: String,
    display_name: Option<String>,
}

#[derive(Serialize)]
struct CreateThreadResponse {
    thread_id: Uuid,
    comment_id: Uuid,
}

async fn open_round_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    control_id: Uuid,
) -> Result<Uuid, ApiError> {
    sqlx::query_scalar(
        "SELECT id FROM review_rounds
         WHERE access_control_id = $1 AND status = 'open'
         FOR UPDATE",
    )
    .bind(control_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ApiError::new(
            StatusCode::CONFLICT,
            "no_open_review",
            "This website has no open review round",
        )
    })
}

async fn site_create_thread(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Query(query): Query<SiteQuery>,
    Json(request): Json<CreateThreadRequest>,
) -> Result<Response, ApiError> {
    let viewer = authorize_site_viewer(&state, &cookies, &headers, query.site, &query.return_to)
        .await?;
    validate_page_path(&request.page_path)?;
    validate_anchor(&request.anchor)?;
    let body = validate_body_text(&request.body)?;
    let display_name = validate_display_name(request.display_name.as_deref())?;
    let (identity_id, public_id, display_name, is_owner) = author_binds(&viewer, display_name);
    let mut transaction = state.database.begin().await?;
    let round_id = open_round_for_update(&mut transaction, viewer.control.id).await?;
    let thread_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM review_threads WHERE round_id = $1")
            .bind(round_id)
            .fetch_one(&mut *transaction)
            .await?;
    if thread_count >= MAX_THREADS_PER_ROUND {
        return Err(ApiError::bad_request(
            "This review round has too many comment threads",
        ));
    }
    let thread_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO review_threads (id, round_id, page_path, anchor)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(thread_id)
    .bind(round_id)
    .bind(&request.page_path)
    .bind(&request.anchor)
    .execute(&mut *transaction)
    .await?;
    let comment_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO review_comments (
            id, thread_id, author_identity_id, author_public_id,
            author_display_name, author_is_owner, body
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(comment_id)
    .bind(thread_id)
    .bind(identity_id)
    .bind(public_id)
    .bind(display_name)
    .bind(is_owner)
    .bind(body)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(no_store(
        (
            StatusCode::CREATED,
            Json(CreateThreadResponse {
                thread_id,
                comment_id,
            }),
        )
            .into_response(),
    ))
}

#[derive(Deserialize)]
struct ReplyRequest {
    body: String,
    display_name: Option<String>,
}

#[derive(Serialize)]
struct ReplyResponse {
    comment_id: Uuid,
}

async fn site_reply(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(thread_id): Path<Uuid>,
    Query(query): Query<SiteQuery>,
    Json(request): Json<ReplyRequest>,
) -> Result<Response, ApiError> {
    let viewer = authorize_site_viewer(&state, &cookies, &headers, query.site, &query.return_to)
        .await?;
    let body = validate_body_text(&request.body)?;
    let display_name = validate_display_name(request.display_name.as_deref())?;
    let (identity_id, public_id, display_name, is_owner) = author_binds(&viewer, display_name);
    let mut transaction = state.database.begin().await?;
    let known_thread: Option<Uuid> = sqlx::query_scalar(
        "SELECT review_threads.id FROM review_threads
         JOIN review_rounds ON review_rounds.id = review_threads.round_id
         WHERE review_threads.id = $1
           AND review_rounds.access_control_id = $2
           AND review_rounds.status = 'open'
         FOR UPDATE OF review_threads",
    )
    .bind(thread_id)
    .bind(viewer.control.id)
    .fetch_optional(&mut *transaction)
    .await?;
    if known_thread.is_none() {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "no_open_review",
            "This comment thread does not belong to an open review round",
        ));
    }
    let comment_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM review_comments WHERE thread_id = $1")
            .bind(thread_id)
            .fetch_one(&mut *transaction)
            .await?;
    if comment_count >= MAX_COMMENTS_PER_THREAD {
        return Err(ApiError::bad_request(
            "This comment thread has too many replies",
        ));
    }
    let comment_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO review_comments (
            id, thread_id, author_identity_id, author_public_id,
            author_display_name, author_is_owner, body
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(comment_id)
    .bind(thread_id)
    .bind(identity_id)
    .bind(public_id)
    .bind(display_name)
    .bind(is_owner)
    .bind(body)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(no_store(
        (StatusCode::CREATED, Json(ReplyResponse { comment_id })).into_response(),
    ))
}

async fn site_finish(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Query(query): Query<SiteQuery>,
) -> Result<Response, ApiError> {
    let viewer = authorize_site_viewer(&state, &cookies, &headers, query.site, &query.return_to)
        .await?;
    let finished: Option<Uuid> = sqlx::query_scalar(
        "UPDATE review_rounds SET status = 'finished', finished_at = now()
         WHERE access_control_id = $1 AND status = 'open'
         RETURNING id",
    )
    .bind(viewer.control.id)
    .fetch_optional(&state.database)
    .await?;
    if finished.is_none() {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "no_open_review",
            "This website has no open review round",
        ));
    }
    Ok(no_store(StatusCode::NO_CONTENT.into_response()))
}

async fn owned_plan_control(
    state: &AppState,
    user: &AuthUser,
    selector: &str,
) -> Result<Uuid, ApiError> {
    sqlx::query_scalar(
        "SELECT access_control_id FROM plans
         WHERE user_id = $1 AND status = 'active'
           AND (id::text = $2 OR slug = $2)",
    )
    .bind(user.id)
    .bind(selector)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(ApiError::not_found)
}

async fn owned_deployment_control(
    state: &AppState,
    user: &AuthUser,
    selector: &str,
) -> Result<Uuid, ApiError> {
    sqlx::query_scalar(
        "SELECT access_control_id FROM deployments
         WHERE user_id = $1 AND status = 'active'
           AND (id::text = $2 OR slug = $2)",
    )
    .bind(user.id)
    .bind(selector)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(ApiError::not_found)
}

#[derive(Deserialize)]
struct RoundQuery {
    round: Option<i32>,
}

async fn review_status(state: &AppState, control_id: Uuid) -> Result<Response, ApiError> {
    let round = latest_round(state, control_id).await?;
    Ok(Json(ReviewStatusResponse { round }).into_response())
}

async fn review_comments(
    state: &AppState,
    control_id: Uuid,
    round_number: Option<i32>,
) -> Result<Response, ApiError> {
    let round = match round_number {
        Some(number) => round_by_number(state, control_id, number).await?,
        None => latest_round(state, control_id).await?,
    }
    .ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "This website has no review round",
        )
    })?;
    Ok(Json(comments_response(state, round).await?).into_response())
}

async fn review_rounds(state: &AppState, control_id: Uuid) -> Result<Response, ApiError> {
    let rows = sqlx::query(&format!(
        "{ROUND_QUERY} WHERE access_control_id = $1 ORDER BY round_number DESC"
    ))
    .bind(control_id)
    .fetch_all(&state.database)
    .await?;
    let rounds = rows
        .iter()
        .map(round_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(ReviewRoundsResponse { rounds }).into_response())
}

async fn plan_review_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
) -> Result<Response, ApiError> {
    let control_id = owned_plan_control(&state, &user, &selector).await?;
    review_status(&state, control_id).await
}

async fn plan_review_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
    Query(query): Query<RoundQuery>,
) -> Result<Response, ApiError> {
    let control_id = owned_plan_control(&state, &user, &selector).await?;
    review_comments(&state, control_id, query.round).await
}

async fn plan_review_rounds(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
) -> Result<Response, ApiError> {
    let control_id = owned_plan_control(&state, &user, &selector).await?;
    review_rounds(&state, control_id).await
}

async fn deployment_review_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
) -> Result<Response, ApiError> {
    let control_id = owned_deployment_control(&state, &user, &selector).await?;
    review_status(&state, control_id).await
}

async fn deployment_review_comments(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
    Query(query): Query<RoundQuery>,
) -> Result<Response, ApiError> {
    let control_id = owned_deployment_control(&state, &user, &selector).await?;
    review_comments(&state, control_id, query.round).await
}

async fn deployment_review_rounds(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
) -> Result<Response, ApiError> {
    let control_id = owned_deployment_control(&state, &user, &selector).await?;
    review_rounds(&state, control_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_text_anchor() {
        let anchor = serde_json::json!({
            "type": "text",
            "exact": "suis du texte",
            "prefix": "je ",
            "suffix": ""
        });
        assert!(validate_anchor(&anchor).is_err());
        let anchor = serde_json::json!({
            "type": "text",
            "exact": "suis du texte",
            "prefix": "je "
        });
        assert!(validate_anchor(&anchor).is_ok());
    }

    #[test]
    fn accepts_valid_element_anchor() {
        let anchor = serde_json::json!({
            "type": "element",
            "css": "main > figure:nth-of-type(2) > img",
            "tag": "img",
            "src": "/assets/diagram.png"
        });
        assert!(validate_anchor(&anchor).is_ok());
    }

    #[test]
    fn rejects_unknown_anchor_types_and_oversized_fields() {
        assert!(validate_anchor(&serde_json::json!({"type": "vibe"})).is_err());
        assert!(validate_anchor(&serde_json::json!("text")).is_err());
        let oversized = serde_json::json!({
            "type": "text",
            "exact": "x".repeat(MAX_ANCHOR_EXACT_CHARS + 1)
        });
        assert!(validate_anchor(&oversized).is_err());
    }

    #[test]
    fn validates_comment_bodies_and_names() {
        assert!(validate_body_text("  ").is_err());
        assert_eq!(validate_body_text("  ok  ").unwrap(), "ok");
        assert!(validate_body_text(&"x".repeat(MAX_BODY_CHARS + 1)).is_err());
        assert_eq!(validate_display_name(Some("  ")).unwrap(), None);
        assert_eq!(
            validate_display_name(Some(" Paul ")).unwrap().as_deref(),
            Some("Paul")
        );
        assert!(validate_display_name(Some(&"x".repeat(65))).is_err());
    }

    #[test]
    fn validates_page_paths() {
        assert!(validate_page_path("/").is_ok());
        assert!(validate_page_path("/architecture").is_ok());
        assert!(validate_page_path("architecture").is_err());
    }
}
