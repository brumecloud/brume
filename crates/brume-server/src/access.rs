use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Form, Json, Router,
    body::Body,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use brume_core::{AuthMode, CreateInvitationResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use tower_cookies::{
    Cookies,
    cookie::{Cookie, SameSite, time},
};
use url::Url;
use uuid::Uuid;

use crate::{
    auth,
    error::ApiError,
    state::AppState,
    util::{hash_secret, random_public_id, random_token},
};

const IDENTITY_COOKIE: &str = "brume_identity";
const SESSION_LIFETIME: Duration = Duration::days(1);
const INVITATION_LIFETIME: Duration = Duration::days(1);
const TICKET_LIFETIME: Duration = Duration::minutes(2);
const GEIST_FONT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/geist.woff2"));
const OVERLAY_SCRIPT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/overlay.js"));

#[derive(Debug, Clone)]
pub struct AccessControl {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub auth_mode: AuthMode,
    pub password_hash: Option<String>,
    pub access_version: i64,
    pub overlay_enabled: bool,
}

pub enum RequestAuthorization {
    Allowed,
    Redirect(String),
}

#[derive(Clone)]
pub struct SiteLocation {
    pub origin: String,
    base_url: String,
    cookie_path: String,
}

#[derive(Clone, Copy)]
enum Subject {
    Owner,
    Token(Uuid),
    Password,
}

impl Subject {
    fn kind(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Token(_) => "token",
            Self::Password => "password",
        }
    }

    fn identity_id(self) -> Option<Uuid> {
        match self {
            Self::Token(identity_id) => Some(identity_id),
            Self::Owner | Self::Password => None,
        }
    }
}

pub fn auth_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/_brume/geist.woff2", get(geist_font))
        .route("/access", get(access_landing))
        .route("/access/token", post(submit_token))
        .route("/access/password", post(submit_password))
        .route("/invitations/{invitation}", get(invitation_landing))
        .route("/invitations/{invitation}/claim", post(claim_invitation))
        .nest("/toolbar/{control_id}", toolbar_router(state))
}

fn toolbar_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/owner-state", get(toolbar_owner_state))
        .route("/access", get(toolbar_access))
        .route("/settings", post(update_toolbar_settings))
        .route("/invitations", post(create_invitation))
        .route("/grants/revoke", post(revoke_grant))
        .layer(middleware::from_fn_with_state(state, toolbar_cors))
}

pub fn site_router() -> Router<AppState> {
    Router::new()
        .route("/_brume/access/complete", get(complete_site_auth_handler))
        .route("/_brume/overlay-state", get(overlay_state))
        .route("/_brume/overlay.js", get(overlay_script))
}

async fn overlay_script(headers: HeaderMap) -> Response {
    // Content hash, not the commit SHA: dirty-tree rebuilds keep the same
    // commit, and a stable ETag would let browsers 304 into a stale bundle.
    static OVERLAY_ETAG: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        use sha2::{Digest, Sha256};
        format!("\"{}\"", hex::encode(Sha256::digest(OVERLAY_SCRIPT)))
    });
    let etag = OVERLAY_ETAG.clone();
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == etag)
    {
        return StatusCode::NOT_MODIFIED.into_response();
    }
    let mut response = Response::new(Body::from(OVERLAY_SCRIPT));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );
    if let Ok(value) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, value);
    }
    response
}

async fn geist_font() -> Response {
    let mut response = Response::new(Body::from(GEIST_FONT));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("font/woff2"));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("cross-origin"),
    );
    response
}

pub async fn insert_control(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    owner_user_id: Uuid,
    auth_mode: AuthMode,
    password: Option<&str>,
    overlay_enabled: bool,
) -> Result<Uuid, ApiError> {
    validate_password_input(auth_mode, password)?;
    let password_hash = match password {
        Some(password) => Some(hash_password(password.to_owned()).await?),
        None => None,
    };
    sqlx::query(
        "INSERT INTO access_controls (
            id, owner_user_id, auth_mode, password_hash, overlay_enabled
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(owner_user_id)
    .bind(auth_mode.to_string())
    .bind(password_hash)
    .bind(overlay_enabled)
    .execute(&mut **transaction)
    .await?;
    Ok(id)
}

pub async fn update_control(
    transaction: &mut Transaction<'_, Postgres>,
    control_id: Uuid,
    owner_user_id: Uuid,
    auth_mode: AuthMode,
    password: Option<&str>,
    overlay_enabled: bool,
) -> Result<(), ApiError> {
    if let Some(password) = password {
        validate_password_input(AuthMode::Password, Some(password))?;
    }
    let current = sqlx::query(
        "SELECT auth_mode, password_hash
         FROM access_controls
         WHERE id = $1 AND owner_user_id = $2
         FOR UPDATE",
    )
    .bind(control_id)
    .bind(owner_user_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(ApiError::not_found)?;
    let current_mode: AuthMode = current
        .try_get::<String, _>("auth_mode")?
        .parse()
        .map_err(ApiError::internal)?;
    let current_password: Option<String> = current.try_get("password_hash")?;
    let password_hash = match (auth_mode, password) {
        (AuthMode::Password, Some(password)) => Some(hash_password(password.to_owned()).await?),
        (AuthMode::Password, None) if current_mode == AuthMode::Password => current_password,
        (AuthMode::Password, None) => {
            return Err(ApiError::bad_request(
                "A password is required when enabling password authentication",
            ));
        }
        _ => None,
    };
    let access_changed = current_mode != auth_mode || password.is_some();
    sqlx::query(
        "UPDATE access_controls SET
            auth_mode = $1,
            password_hash = $2,
            overlay_enabled = $3,
            access_version = access_version + CASE WHEN $4 THEN 1 ELSE 0 END,
            updated_at = now()
         WHERE id = $5",
    )
    .bind(auth_mode.to_string())
    .bind(password_hash)
    .bind(overlay_enabled)
    .bind(access_changed)
    .bind(control_id)
    .execute(&mut **transaction)
    .await?;
    if access_changed {
        sqlx::query(
            "UPDATE access_invitations SET consumed_at = now()
             WHERE access_control_id = $1 AND consumed_at IS NULL",
        )
        .bind(control_id)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

pub async fn patch_control(
    state: &AppState,
    control_id: Uuid,
    owner_user_id: Uuid,
    auth_mode: Option<AuthMode>,
    password: Option<&str>,
    overlay_enabled: Option<bool>,
) -> Result<AccessControl, ApiError> {
    let current = load_control(state, control_id).await?;
    if current.owner_user_id != owner_user_id {
        return Err(ApiError::not_found());
    }
    let mut transaction = state.database.begin().await?;
    update_control(
        &mut transaction,
        control_id,
        owner_user_id,
        auth_mode.unwrap_or(current.auth_mode),
        password,
        overlay_enabled.unwrap_or(current.overlay_enabled),
    )
    .await?;
    transaction.commit().await?;
    load_control(state, control_id).await
}

pub async fn load_control(state: &AppState, control_id: Uuid) -> Result<AccessControl, ApiError> {
    let row = sqlx::query(
        "SELECT id, owner_user_id, auth_mode, password_hash,
                access_version, overlay_enabled
         FROM access_controls WHERE id = $1",
    )
    .bind(control_id)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(ApiError::not_found)?;
    control_from_row(&row)
}

fn control_from_row(row: &sqlx::postgres::PgRow) -> Result<AccessControl, ApiError> {
    Ok(AccessControl {
        id: row.try_get("id")?,
        owner_user_id: row.try_get("owner_user_id")?,
        auth_mode: row
            .try_get::<String, _>("auth_mode")?
            .parse()
            .map_err(ApiError::internal)?,
        password_hash: row.try_get("password_hash")?,
        access_version: row.try_get("access_version")?,
        overlay_enabled: row.try_get("overlay_enabled")?,
    })
}

pub async fn authorize_request(
    state: &AppState,
    cookies: &Cookies,
    control: &AccessControl,
    return_to: &str,
    owner_authenticated: bool,
    bearer_token: Option<&str>,
) -> Result<RequestAuthorization, ApiError> {
    if control.auth_mode == AuthMode::None || owner_authenticated {
        return Ok(RequestAuthorization::Allowed);
    }
    if control.auth_mode == AuthMode::Token
        && let Some(token) = bearer_token
        && let Some(identity) = identity_from_token_optional(state, token).await?
        && grant_exists(state, control.id, identity.id).await?
    {
        return Ok(RequestAuthorization::Allowed);
    }
    if let Some(cookie) = cookies.get(&site_cookie_name(control.id)) {
        let row = sqlx::query(
            "SELECT subject_kind, identity_id
             FROM site_sessions
             WHERE access_control_id = $1
               AND session_hash = $2
               AND access_version = $3
               AND expires_at > now()",
        )
        .bind(control.id)
        .bind(hash_secret(cookie.value()))
        .bind(control.access_version)
        .fetch_optional(&state.database)
        .await?;
        if let Some(row) = row {
            let kind: String = row.try_get("subject_kind")?;
            let identity_id: Option<Uuid> = row.try_get("identity_id")?;
            let still_allowed = match (kind.as_str(), identity_id) {
                ("token", Some(identity_id)) => {
                    grant_exists(state, control.id, identity_id).await?
                }
                ("owner", _) | ("password", _) => true,
                _ => false,
            };
            if still_allowed {
                return Ok(RequestAuthorization::Allowed);
            }
        }
    }
    Ok(RequestAuthorization::Redirect(format!(
        "{}/access?site={}&return_to={}",
        state.config.auth_public_url,
        control.id,
        urlencoding::encode(return_to)
    )))
}

pub async fn site_session_is_owner(
    state: &AppState,
    cookies: &Cookies,
    control: &AccessControl,
) -> Result<bool, ApiError> {
    let Some(cookie) = cookies.get(&site_cookie_name(control.id)) else {
        return Ok(false);
    };
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM site_sessions
            WHERE access_control_id = $1
              AND session_hash = $2
              AND access_version = $3
              AND subject_kind = 'owner'
              AND expires_at > now()
        )",
    )
    .bind(control.id)
    .bind(hash_secret(cookie.value()))
    .bind(control.access_version)
    .fetch_one(&state.database)
    .await?)
}

pub fn overlay_markup(control_id: Uuid, auth_origin: &str, nonce: Option<&str>) -> String {
    let nonce = nonce
        .map(|value| format!(" nonce=\"{}\"", escape_html(value)))
        .unwrap_or_default();
    format!(
        r#"<style data-brume-overlay-style>@font-face{{font-family:"Geist Variable";font-style:normal;font-weight:100 900;font-display:swap;src:url("{auth_origin}/_brume/geist.woff2") format("woff2")}}[data-brume-overlay-host]{{all:initial;position:fixed!important;right:16px!important;bottom:16px!important;z-index:2147483647!important;color-scheme:dark!important}}[data-brume-overlay-host][data-side="left"]{{right:auto!important;left:16px!important}}[data-brume-review-host]{{all:initial;position:absolute!important;top:0!important;left:0!important;width:0!important;height:0!important;overflow:visible!important;z-index:2147483646!important;color-scheme:dark!important}}</style><script src="/_brume/overlay.js" defer{nonce} data-brume-overlay-script data-brume-site="{control_id}" data-brume-auth-origin="{auth_origin}"></script>"#,
        auth_origin = escape_html(auth_origin),
    )
}

#[derive(Deserialize)]
struct OverlayStateQuery {
    site: Uuid,
    return_to: String,
}

#[derive(Serialize)]
struct OverlayState {
    enabled: bool,
    owner: bool,
    identified: bool,
    /// Present only for the owner: how the site is currently shared.
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review: Option<crate::review::ReviewOverlayState>,
}

async fn overlay_state(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<OverlayStateQuery>,
) -> Result<Response, ApiError> {
    let control = load_control(&state, query.site).await?;
    validate_return_to(&state, control.id, &query.return_to).await?;
    let owner = site_session_is_owner(&state, &cookies, &control).await?
        || auth::web_user(&state, &cookies)
            .await?
            .is_some_and(|user| user.id == control.owner_user_id);
    let identified = identity_from_cookie(&state, &cookies).await?.is_some();
    let review = crate::review::overlay_review_state(&state, control.id).await?;
    let mut response = Json(OverlayState {
        enabled: control.overlay_enabled,
        owner,
        identified,
        auth_mode: owner.then(|| control.auth_mode.to_string()),
        review,
    })
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
}

/// CORS gate for every `/toolbar/{control_id}` endpoint. The overlay calls
/// these cross-origin from the site it is injected into, so the request
/// `Origin` must match the control's own canonical site origin, derived
/// server-side. That origin check doubles as CSRF protection: same-site
/// cookies are attached automatically, but no other Brume-hosted site can
/// forge a matching `Origin` header for someone else's control.
async fn toolbar_cors(
    State(state): State<AppState>,
    Path(control_id): Path<Uuid>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let location = site_location(&state, control_id).await?;
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if origin != Some(location.origin.as_str()) {
        return Err(ApiError::forbidden("Invalid management request origin"));
    }
    let allow_origin = HeaderValue::from_str(&location.origin).map_err(ApiError::internal)?;
    if request.method() == Method::OPTIONS {
        let mut response = StatusCode::NO_CONTENT.into_response();
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, allow_origin);
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            HeaderValue::from_static("true"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("content-type"),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            HeaderValue::from_static("600"),
        );
        headers.insert(header::VARY, HeaderValue::from_static("Origin"));
        return Ok(response);
    }
    request.extensions_mut().insert(location);
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, allow_origin);
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    headers.insert(header::VARY, HeaderValue::from_static("Origin"));
    Ok(response)
}

#[derive(Serialize)]
struct ToolbarOwnerState {
    owner: bool,
}

async fn toolbar_owner_state(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
) -> Result<Json<ToolbarOwnerState>, ApiError> {
    let control = load_control(&state, control_id).await?;
    let owner = auth::browser_user(&state, &cookies)
        .await?
        .is_some_and(|user| user.id == control.owner_user_id);
    Ok(Json(ToolbarOwnerState { owner }))
}

#[derive(Serialize)]
struct GrantSummary {
    public_id: String,
    display_name: Option<String>,
    granted_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct ToolbarAccess {
    auth_mode: String,
    has_password: bool,
    grants: Vec<GrantSummary>,
}

async fn list_grants(state: &AppState, control_id: Uuid) -> Result<Vec<GrantSummary>, ApiError> {
    let rows = sqlx::query(
        "SELECT access_identities.public_id,
                access_grants.created_at AS granted_at,
                access_identities.last_used_at,
                (SELECT review_comments.author_display_name
                 FROM review_comments
                 JOIN review_threads ON review_threads.id = review_comments.thread_id
                 JOIN review_rounds ON review_rounds.id = review_threads.round_id
                 WHERE review_comments.author_identity_id = access_identities.id
                   AND review_rounds.access_control_id = access_grants.access_control_id
                   AND review_comments.author_display_name IS NOT NULL
                 ORDER BY review_comments.created_at DESC
                 LIMIT 1) AS display_name
         FROM access_grants
         JOIN access_identities ON access_identities.id = access_grants.identity_id
         WHERE access_grants.access_control_id = $1
           AND access_identities.revoked_at IS NULL
         ORDER BY access_grants.created_at DESC",
    )
    .bind(control_id)
    .fetch_all(&state.database)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(GrantSummary {
                public_id: row.try_get("public_id")?,
                display_name: row.try_get("display_name")?,
                granted_at: row.try_get("granted_at")?,
                last_used_at: row.try_get("last_used_at")?,
            })
        })
        .collect()
}

async fn toolbar_access_payload(
    state: &AppState,
    control: &AccessControl,
) -> Result<ToolbarAccess, ApiError> {
    Ok(ToolbarAccess {
        auth_mode: control.auth_mode.to_string(),
        has_password: control.password_hash.is_some(),
        grants: list_grants(state, control.id).await?,
    })
}

async fn toolbar_access(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
) -> Result<Json<ToolbarAccess>, ApiError> {
    let (_, control) = require_toolbar_owner(&state, &cookies, control_id).await?;
    Ok(Json(toolbar_access_payload(&state, &control).await?))
}

#[derive(Deserialize)]
struct AccessQuery {
    site: Uuid,
    return_to: String,
    error: Option<String>,
}

async fn access_landing(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<AccessQuery>,
) -> Result<Response, ApiError> {
    let control = load_control(&state, query.site).await?;
    let location = validate_return_to(&state, control.id, &query.return_to).await?;
    if control.auth_mode == AuthMode::None {
        return Ok(Redirect::to(&query.return_to).into_response());
    }
    if auth::browser_user(&state, &cookies)
        .await?
        .is_some_and(|user| user.id == control.owner_user_id)
    {
        return issue_ticket(
            &state,
            &control,
            Subject::Owner,
            &query.return_to,
            &location,
        )
        .await
        .map(IntoResponse::into_response);
    }
    if control.auth_mode == AuthMode::Token
        && let Some(identity) = identity_from_cookie(&state, &cookies).await?
        && grant_exists(&state, control.id, identity.id).await?
    {
        return issue_ticket(
            &state,
            &control,
            Subject::Token(identity.id),
            &query.return_to,
            &location,
        )
        .await
        .map(IntoResponse::into_response);
    }
    Ok(Html(access_page(&state, &control, &query)).into_response())
}

#[derive(Deserialize)]
struct AccessForm {
    site: Uuid,
    return_to: String,
    token: Option<String>,
    password: Option<String>,
}

async fn submit_token(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Form(form): Form<AccessForm>,
) -> Result<Response, ApiError> {
    require_auth_origin(&state, &headers)?;
    let control = load_control(&state, form.site).await?;
    if control.auth_mode != AuthMode::Token {
        return Err(ApiError::bad_request(
            "This website does not use token authentication",
        ));
    }
    let location = validate_return_to(&state, control.id, &form.return_to).await?;
    let token = form.token.unwrap_or_default();
    let identity = identity_from_token(&state, token.trim()).await?;
    if !grant_exists(&state, control.id, identity.id).await? {
        return Ok(access_error_redirect(
            &state,
            control.id,
            &form.return_to,
            "This token does not have access to this website",
        ));
    }
    set_identity_cookie(&state, &cookies, token.trim());
    issue_ticket(
        &state,
        &control,
        Subject::Token(identity.id),
        &form.return_to,
        &location,
    )
    .await
    .map(IntoResponse::into_response)
}

async fn submit_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AccessForm>,
) -> Result<Response, ApiError> {
    require_auth_origin(&state, &headers)?;
    let control = load_control(&state, form.site).await?;
    if control.auth_mode != AuthMode::Password {
        return Err(ApiError::bad_request(
            "This website does not use password authentication",
        ));
    }
    let location = validate_return_to(&state, control.id, &form.return_to).await?;
    let client_hash = password_client_hash(control.id, &headers);
    let global_hash = hash_secret(&format!("{}:global-password-attempts", control.id));
    ensure_password_attempt_allowed(&state, control.id, &client_hash).await?;
    ensure_password_attempt_allowed(&state, control.id, &global_hash).await?;
    let valid = match (&control.password_hash, form.password.as_deref()) {
        (Some(expected), Some(password)) => {
            verify_password(expected.to_owned(), password.to_owned()).await?
        }
        _ => false,
    };
    if !valid {
        record_password_failure(&state, control.id, &client_hash, 10).await?;
        record_password_failure(&state, control.id, &global_hash, 1_000).await?;
        return Ok(access_error_redirect(
            &state,
            control.id,
            &form.return_to,
            "Incorrect password",
        ));
    }
    clear_password_failures(&state, control.id, &client_hash).await?;
    issue_ticket(
        &state,
        &control,
        Subject::Password,
        &form.return_to,
        &location,
    )
    .await
    .map(IntoResponse::into_response)
}

#[derive(Deserialize)]
pub struct TicketQuery {
    ticket: String,
}

pub async fn complete_site_auth(
    state: AppState,
    cookies: Cookies,
    query: TicketQuery,
    current_origin: String,
) -> Result<Response, ApiError> {
    let mut transaction = state.database.begin().await?;
    let row = sqlx::query(
        "SELECT id, access_control_id, identity_id, subject_kind,
                completion_origin, return_to, cookie_path
         FROM site_auth_tickets
         WHERE ticket_hash = $1 AND expires_at > now() AND consumed_at IS NULL
         FOR UPDATE",
    )
    .bind(hash_secret(&query.ticket))
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::bad_request("Authentication ticket is invalid or expired"))?;
    let completion_origin: String = row.try_get("completion_origin")?;
    if completion_origin != current_origin {
        return Err(ApiError::bad_request(
            "Authentication ticket belongs to another website",
        ));
    }
    let ticket_id: Uuid = row.try_get("id")?;
    let control_id: Uuid = row.try_get("access_control_id")?;
    let identity_id: Option<Uuid> = row.try_get("identity_id")?;
    let subject_kind: String = row.try_get("subject_kind")?;
    let return_to: String = row.try_get("return_to")?;
    let cookie_path: String = row.try_get("cookie_path")?;
    let access_version: i64 =
        sqlx::query_scalar("SELECT access_version FROM access_controls WHERE id = $1")
            .bind(control_id)
            .fetch_one(&mut *transaction)
            .await?;
    let session = random_token("site_");
    sqlx::query(
        "INSERT INTO site_sessions (
            id, access_control_id, identity_id, subject_kind,
            session_hash, access_version, expires_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(Uuid::now_v7())
    .bind(control_id)
    .bind(identity_id)
    .bind(subject_kind)
    .bind(hash_secret(&session))
    .bind(access_version)
    .bind(Utc::now() + SESSION_LIFETIME)
    .execute(&mut *transaction)
    .await?;
    sqlx::query("UPDATE site_auth_tickets SET consumed_at = now() WHERE id = $1")
        .bind(ticket_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    cookies.add(
        Cookie::build((site_cookie_name(control_id), session))
            .path(cookie_path)
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(current_origin.starts_with("https://"))
            .max_age(time::Duration::days(1))
            .build(),
    );
    Ok(Redirect::to(&return_to).into_response())
}

async fn complete_site_auth_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Query(query): Query<TicketQuery>,
) -> Result<Response, ApiError> {
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::bad_request("Missing website host"))?;
    let current_origin = if host.eq_ignore_ascii_case(&state.config.plan_public_host) {
        state.config.plan_public_url.clone()
    } else {
        let label = host
            .split(':')
            .next()
            .and_then(|host| host.strip_suffix(&format!(".{}", state.config.public_domain)))
            .filter(|label| !label.is_empty() && !label.contains('.'))
            .ok_or_else(|| ApiError::bad_request("Invalid website host"))?;
        state.config.public_url(label)
    };
    complete_site_auth(state, cookies, query, current_origin).await
}

#[derive(Deserialize)]
struct SettingsRequest {
    auth: AuthMode,
    password: Option<String>,
}

async fn update_toolbar_settings(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Json(body): Json<SettingsRequest>,
) -> Result<Json<ToolbarAccess>, ApiError> {
    let (user, _) = require_toolbar_owner(&state, &cookies, control_id).await?;
    let control = patch_control(
        &state,
        control_id,
        user.id,
        Some(body.auth),
        body.password.as_deref().filter(|value| !value.is_empty()),
        None,
    )
    .await?;
    Ok(Json(toolbar_access_payload(&state, &control).await?))
}

#[derive(Deserialize)]
struct InvitationRequest {
    return_to: Option<String>,
}

async fn create_invitation(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Extension(location): Extension<SiteLocation>,
    Json(body): Json<InvitationRequest>,
) -> Result<Json<CreateInvitationResponse>, ApiError> {
    require_toolbar_owner(&state, &cookies, control_id).await?;
    let return_to = match body.return_to {
        Some(value) => {
            validate_return_to(&state, control_id, &value).await?;
            value
        }
        None => location.base_url.clone(),
    };
    let invitation = random_token("invite_");
    let expires_at = Utc::now() + INVITATION_LIFETIME;
    sqlx::query(
        "INSERT INTO access_invitations (
            id, access_control_id, token_hash, return_to, expires_at
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(control_id)
    .bind(hash_secret(&invitation))
    .bind(&return_to)
    .bind(expires_at)
    .execute(&state.database)
    .await?;
    Ok(Json(CreateInvitationResponse {
        url: format!(
            "{}/invitations/{}",
            state.config.auth_public_url, invitation
        ),
        expires_at,
    }))
}

#[derive(Deserialize)]
struct RevokeRequest {
    public_id: String,
}

#[derive(Serialize)]
struct GrantsResponse {
    grants: Vec<GrantSummary>,
}

async fn revoke_grant(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Json(body): Json<RevokeRequest>,
) -> Result<Json<GrantsResponse>, ApiError> {
    require_toolbar_owner(&state, &cookies, control_id).await?;
    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM access_identities
         WHERE public_id = $1 AND revoked_at IS NULL",
    )
    .bind(body.public_id.trim())
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(|| ApiError::bad_request("Unknown recipient public ID"))?;
    let mut transaction = state.database.begin().await?;
    sqlx::query(
        "DELETE FROM access_grants
         WHERE access_control_id = $1 AND identity_id = $2",
    )
    .bind(control_id)
    .bind(identity_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "DELETE FROM site_sessions
         WHERE access_control_id = $1 AND identity_id = $2",
    )
    .bind(control_id)
    .bind(identity_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Json(GrantsResponse {
        grants: list_grants(&state, control_id).await?,
    }))
}

#[derive(Deserialize)]
struct InvitationQuery {
    return_to: Option<String>,
}

async fn invitation_landing(
    State(state): State<AppState>,
    Path(invitation): Path<String>,
    Query(query): Query<InvitationQuery>,
) -> Result<Html<String>, ApiError> {
    let (_, stored_return_to) = invitation_control(&state, &invitation).await?;
    let return_to = query.return_to.unwrap_or(stored_return_to);
    Ok(Html(invitation_page(&invitation, &return_to)))
}

#[derive(Deserialize)]
struct InvitationForm {
    return_to: Option<String>,
    token: Option<String>,
}

async fn claim_invitation(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(invitation): Path<String>,
    Form(form): Form<InvitationForm>,
) -> Result<Html<String>, ApiError> {
    require_auth_origin(&state, &headers)?;
    let mut transaction = state.database.begin().await?;
    let row = sqlx::query(
        "SELECT id, access_control_id, return_to
         FROM access_invitations
         WHERE token_hash = $1 AND expires_at > now() AND consumed_at IS NULL
         FOR UPDATE",
    )
    .bind(hash_secret(&invitation))
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::bad_request("Invitation is invalid or expired"))?;
    let invitation_id: Uuid = row.try_get("id")?;
    let control_id: Uuid = row.try_get("access_control_id")?;
    let return_to: String = row.try_get("return_to")?;
    let existing_identity = identity_from_cookie_tx(&mut transaction, &cookies).await?;
    let submitted_token = form
        .token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let (identity_id, public_id, new_token, cookie_token) =
        if let Some(identity) = existing_identity {
            (identity.id, identity.public_id, None, None)
        } else if let Some(token) = submitted_token {
            let row = sqlx::query(
                "SELECT id, public_id FROM access_identities
             WHERE token_hash = $1 AND revoked_at IS NULL",
            )
            .bind(hash_secret(token))
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or_else(|| ApiError::bad_request("Access token is invalid or revoked"))?;
            (
                row.try_get("id")?,
                row.try_get("public_id")?,
                None,
                Some(token.to_owned()),
            )
        } else {
            let identity_id = Uuid::now_v7();
            let public_id = format!("brume_{}", random_public_id());
            let token = random_token("brume_user_");
            sqlx::query(
                "INSERT INTO access_identities (id, public_id, token_hash)
             VALUES ($1, $2, $3)",
            )
            .bind(identity_id)
            .bind(&public_id)
            .bind(hash_secret(&token))
            .execute(&mut *transaction)
            .await?;
            (identity_id, public_id, Some(token.clone()), Some(token))
        };
    sqlx::query(
        "INSERT INTO access_grants (access_control_id, identity_id)
         VALUES ($1, $2)
         ON CONFLICT (access_control_id, identity_id) DO NOTHING",
    )
    .bind(control_id)
    .bind(identity_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query("UPDATE access_invitations SET consumed_at = now() WHERE id = $1")
        .bind(invitation_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    if let Some(token) = &cookie_token {
        set_identity_cookie(&state, &cookies, token);
    }
    let destination = form
        .return_to
        .filter(|value| value == &return_to)
        .unwrap_or(return_to);
    Ok(Html(claimed_invitation_page(
        &public_id,
        new_token.as_deref(),
        &destination,
    )))
}

pub struct Identity {
    pub id: Uuid,
    pub public_id: String,
}

pub async fn identity_from_cookie(
    state: &AppState,
    cookies: &Cookies,
) -> Result<Option<Identity>, ApiError> {
    let Some(cookie) = cookies.get(IDENTITY_COOKIE) else {
        return Ok(None);
    };
    identity_from_token_optional(state, cookie.value()).await
}

async fn identity_from_cookie_tx(
    transaction: &mut Transaction<'_, Postgres>,
    cookies: &Cookies,
) -> Result<Option<Identity>, ApiError> {
    let Some(cookie) = cookies.get(IDENTITY_COOKIE) else {
        return Ok(None);
    };
    let row = sqlx::query(
        "SELECT id, public_id FROM access_identities
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(hash_secret(cookie.value()))
    .fetch_optional(&mut **transaction)
    .await?;
    row.map(|row| {
        Ok(Identity {
            id: row.try_get("id")?,
            public_id: row.try_get("public_id")?,
        })
    })
    .transpose()
}

async fn identity_from_token(state: &AppState, token: &str) -> Result<Identity, ApiError> {
    identity_from_token_optional(state, token)
        .await?
        .ok_or_else(ApiError::unauthorized)
}

async fn identity_from_token_optional(
    state: &AppState,
    token: &str,
) -> Result<Option<Identity>, ApiError> {
    let row = sqlx::query(
        "SELECT id, public_id FROM access_identities
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(hash_secret(token))
    .fetch_optional(&state.database)
    .await?;
    let identity = row
        .map(|row| {
            Ok::<_, ApiError>(Identity {
                id: row.try_get("id")?,
                public_id: row.try_get("public_id")?,
            })
        })
        .transpose()?;
    if identity.is_some() {
        let _ = sqlx::query(
            "UPDATE access_identities SET last_used_at = now()
             WHERE token_hash = $1
               AND (last_used_at IS NULL OR last_used_at < now() - interval '1 hour')",
        )
        .bind(hash_secret(token))
        .execute(&state.database)
        .await;
    }
    Ok(identity)
}

async fn grant_exists(
    state: &AppState,
    control_id: Uuid,
    identity_id: Uuid,
) -> Result<bool, ApiError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM access_grants
            JOIN access_identities ON access_identities.id = access_grants.identity_id
            WHERE access_grants.access_control_id = $1
              AND access_grants.identity_id = $2
              AND access_identities.revoked_at IS NULL
        )",
    )
    .bind(control_id)
    .bind(identity_id)
    .fetch_one(&state.database)
    .await?)
}

async fn invitation_control(
    state: &AppState,
    invitation: &str,
) -> Result<(Uuid, String), ApiError> {
    let row = sqlx::query(
        "SELECT access_control_id, return_to FROM access_invitations
         WHERE token_hash = $1 AND expires_at > now() AND consumed_at IS NULL",
    )
    .bind(hash_secret(invitation))
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(|| ApiError::bad_request("Invitation is invalid or expired"))?;
    Ok((row.try_get("access_control_id")?, row.try_get("return_to")?))
}

async fn issue_ticket(
    state: &AppState,
    control: &AccessControl,
    subject: Subject,
    return_to: &str,
    location: &SiteLocation,
) -> Result<Redirect, ApiError> {
    let ticket = random_token("site_ticket_");
    sqlx::query(
        "INSERT INTO site_auth_tickets (
            id, access_control_id, identity_id, subject_kind, ticket_hash,
            completion_origin, return_to, cookie_path, expires_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(Uuid::now_v7())
    .bind(control.id)
    .bind(subject.identity_id())
    .bind(subject.kind())
    .bind(hash_secret(&ticket))
    .bind(&location.origin)
    .bind(return_to)
    .bind(&location.cookie_path)
    .bind(Utc::now() + TICKET_LIFETIME)
    .execute(&state.database)
    .await?;
    Ok(Redirect::to(&format!(
        "{}/_brume/access/complete?ticket={}",
        location.origin,
        urlencoding::encode(&ticket)
    )))
}

pub async fn owner_ticket(
    state: &AppState,
    owner_user_id: Uuid,
    control_id: Uuid,
    return_to: &str,
) -> Result<Redirect, ApiError> {
    let control = load_control(state, control_id).await?;
    if control.owner_user_id != owner_user_id {
        return Err(ApiError::not_found());
    }
    let location = validate_return_to(state, control_id, return_to).await?;
    issue_ticket(state, &control, Subject::Owner, return_to, &location).await
}

async fn site_location(state: &AppState, control_id: Uuid) -> Result<SiteLocation, ApiError> {
    if let Some(row) = sqlx::query(
        "SELECT users.handle, plans.slug
         FROM plans
         JOIN users ON users.id = plans.user_id
         WHERE plans.access_control_id = $1 AND plans.status = 'active'",
    )
    .bind(control_id)
    .fetch_optional(&state.database)
    .await?
    {
        let handle: String = row.try_get("handle")?;
        let slug: String = row.try_get("slug")?;
        let cookie_path = format!("/{handle}/{slug}");
        return Ok(SiteLocation {
            origin: state.config.plan_public_url.clone(),
            base_url: format!("{}{cookie_path}", state.config.plan_public_url),
            cookie_path,
        });
    }
    if let Some(public_label) = sqlx::query_scalar::<_, String>(
        "SELECT public_label FROM deployments
         WHERE access_control_id = $1 AND status = 'active'",
    )
    .bind(control_id)
    .fetch_optional(&state.database)
    .await?
    {
        let origin = state.config.public_url(&public_label);
        return Ok(SiteLocation {
            base_url: format!("{origin}/"),
            origin,
            cookie_path: "/".to_owned(),
        });
    }
    Err(ApiError::not_found())
}

pub async fn validate_return_to(
    state: &AppState,
    control_id: Uuid,
    return_to: &str,
) -> Result<SiteLocation, ApiError> {
    let location = site_location(state, control_id).await?;
    let parsed = Url::parse(return_to).map_err(ApiError::bad_request)?;
    let allowed = Url::parse(&location.base_url).map_err(ApiError::internal)?;
    let allowed_path = allowed.path().trim_end_matches('/');
    let path_allowed = parsed.path() == allowed_path
        || parsed
            .path()
            .strip_prefix(allowed_path)
            .is_some_and(|suffix| suffix.starts_with('/'));
    if parsed.origin() != allowed.origin()
        || !path_allowed
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(ApiError::bad_request("Invalid website return URL"));
    }
    Ok(location)
}

fn site_cookie_name(control_id: Uuid) -> String {
    format!("brume_site_{}", control_id.simple())
}

fn set_identity_cookie(state: &AppState, cookies: &Cookies, token: &str) {
    cookies.add(
        Cookie::build((IDENTITY_COOKIE, token.to_owned()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(state.config.auth_public_url.starts_with("https://"))
            .max_age(time::Duration::days(3650))
            .build(),
    );
}

fn validate_password_input(auth_mode: AuthMode, password: Option<&str>) -> Result<(), ApiError> {
    if auth_mode == AuthMode::Password && password.is_none() {
        return Err(ApiError::bad_request(
            "A password is required for password authentication",
        ));
    }
    if let Some(password) = password
        && !(8..=256).contains(&password.chars().count())
    {
        return Err(ApiError::bad_request(
            "Passwords must contain between 8 and 256 characters",
        ));
    }
    Ok(())
}

async fn hash_password(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|value| value.to_string())
            .map_err(ApiError::internal)
    })
    .await
    .map_err(ApiError::internal)?
}

async fn verify_password(expected: String, password: String) -> Result<bool, ApiError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&expected).map_err(ApiError::internal)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(ApiError::internal)?
}

fn require_auth_origin(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = Url::parse(&state.config.auth_public_url)
        .map_err(ApiError::internal)?
        .origin()
        .ascii_serialization();
    let actual = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if actual != Some(expected.as_str()) {
        return Err(ApiError::forbidden("Invalid authentication request origin"));
    }
    Ok(())
}

fn password_client_hash(control_id: Uuid, headers: &HeaderMap) -> Vec<u8> {
    let address = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .unwrap_or("unknown");
    hash_secret(&format!("{control_id}:{address}"))
}

async fn ensure_password_attempt_allowed(
    state: &AppState,
    control_id: Uuid,
    client_hash: &[u8],
) -> Result<(), ApiError> {
    let blocked: bool = sqlx::query_scalar(
        "SELECT COALESCE(blocked_until > now(), false)
         FROM password_attempts
         WHERE access_control_id = $1 AND client_hash = $2",
    )
    .bind(control_id)
    .bind(client_hash)
    .fetch_optional(&state.database)
    .await?
    .unwrap_or(false);
    if blocked {
        return Err(ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "password_rate_limited",
            "Too many password attempts. Try again later.",
        ));
    }
    Ok(())
}

async fn record_password_failure(
    state: &AppState,
    control_id: Uuid,
    client_hash: &[u8],
    failure_limit: i32,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO password_attempts (
            access_control_id, client_hash, failures, window_started_at
         ) VALUES ($1, $2, 1, now())
         ON CONFLICT (access_control_id, client_hash) DO UPDATE SET
            failures = CASE
                WHEN password_attempts.window_started_at < now() - interval '15 minutes'
                    THEN 1
                ELSE password_attempts.failures + 1
            END,
            window_started_at = CASE
                WHEN password_attempts.window_started_at < now() - interval '15 minutes'
                    THEN now()
                ELSE password_attempts.window_started_at
            END,
            blocked_until = CASE
                WHEN (
                    CASE
                        WHEN password_attempts.window_started_at < now() - interval '15 minutes'
                            THEN 1
                        ELSE password_attempts.failures + 1
                    END
                ) >= $3
                    THEN now() + interval '15 minutes'
                ELSE password_attempts.blocked_until
            END",
    )
    .bind(control_id)
    .bind(client_hash)
    .bind(failure_limit)
    .execute(&state.database)
    .await?;
    Ok(())
}

async fn clear_password_failures(
    state: &AppState,
    control_id: Uuid,
    client_hash: &[u8],
) -> Result<(), ApiError> {
    sqlx::query(
        "DELETE FROM password_attempts
         WHERE access_control_id = $1 AND client_hash = $2",
    )
    .bind(control_id)
    .bind(client_hash)
    .execute(&state.database)
    .await?;
    Ok(())
}

async fn require_toolbar_owner(
    state: &AppState,
    cookies: &Cookies,
    control_id: Uuid,
) -> Result<(auth::AuthUser, AccessControl), ApiError> {
    let user = auth::browser_user(state, cookies)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    let control = load_control(state, control_id).await?;
    if control.owner_user_id != user.id {
        return Err(ApiError::not_found());
    }
    Ok((user, control))
}

fn access_error_redirect(
    state: &AppState,
    control_id: Uuid,
    return_to: &str,
    error: &str,
) -> Response {
    Redirect::to(&format!(
        "{}/access?site={}&return_to={}&error={}",
        state.config.auth_public_url,
        control_id,
        urlencoding::encode(return_to),
        urlencoding::encode(error)
    ))
    .into_response()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn access_page(state: &AppState, control: &AccessControl, query: &AccessQuery) -> String {
    let action = match control.auth_mode {
        AuthMode::Token => "/access/token",
        AuthMode::Password => "/access/password",
        AuthMode::None => "/access",
    };
    let label = match control.auth_mode {
        AuthMode::Token => "Access token",
        AuthMode::Password => "Password",
        AuthMode::None => "",
    };
    let input_type = if control.auth_mode == AuthMode::Password {
        "password"
    } else {
        "text"
    };
    let name = if control.auth_mode == AuthMode::Password {
        "password"
    } else {
        "token"
    };
    let error = query
        .error
        .as_deref()
        .map(|value| format!("<p class=\"error\">{}</p>", escape_html(value)))
        .unwrap_or_default();
    let login = format!(
        "{}/auth/github/start?site={}&site_return_to={}",
        state.config.auth_public_url,
        control.id,
        urlencoding::encode(&query.return_to)
    );
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Open Brume website</title>{AUTH_STYLES}</head><body><main><div class="mark">☁️</div><h1>This website is protected</h1><p>Authenticate before Brume loads its content.</p>{error}<form method="post" action="{action}"><input type="hidden" name="site" value="{site}"><input type="hidden" name="return_to" value="{return_to}"><label>{label}<input autofocus required autocomplete="off" type="{input_type}" name="{name}"></label><button>Continue</button></form><a class="owner" href="{login}">Website owner? Sign in with GitHub</a></main></body></html>"#,
        site = control.id,
        return_to = escape_html(&query.return_to),
    )
}

const AUTH_STYLES: &str = r#"<style>
@font-face{font-family:"Geist Variable";font-style:normal;font-weight:100 900;font-display:swap;src:url("/_brume/geist.woff2") format("woff2")}
:root{color-scheme:light dark;font-family:"Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif;background:#fafafa;color:#171717}
*{box-sizing:border-box}body{margin:0;min-height:100vh;display:grid;place-items:center;padding:24px;font:15px/1.45 "Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif}button,input,select,a{font:inherit}
main{width:min(100%,380px);padding:28px;border:1px solid #e7e7e7;border-radius:18px;background:rgba(255,255,255,.94);box-shadow:0 18px 55px rgba(0,0,0,.12)}
.mark{width:44px;height:44px;display:grid;place-items:center;border-radius:50%;background:#111;margin-bottom:18px}
h1{font-size:21px;margin:0 0 6px}p{margin:0 0 20px;color:#666}label{display:grid;gap:7px;font-weight:600}
input{width:100%;border:1px solid #d5d5d5;border-radius:10px;padding:11px 12px;background:transparent;color:inherit}
button{width:100%;margin-top:12px;padding:11px;border:0;border-radius:10px;background:#171717;color:white;font-weight:650;cursor:pointer}
.owner{display:block;margin-top:18px;text-align:center;color:#666;font-size:13px}.error{color:#b42318;background:#fee4e2;padding:9px 11px;border-radius:9px}
@media(prefers-color-scheme:dark){:root{background:#0a0a0a;color:#ededed}main{background:rgba(20,20,20,.96);border-color:#333}p,.owner{color:#aaa}input{border-color:#444}}
</style>"#;

fn invitation_page(invitation: &str, return_to: &str) -> String {
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Accept Brume invitation</title>{AUTH_STYLES}</head><body><main><div class="mark">☁️</div><h1>Website invitation</h1><p>This one-time invitation adds the website to your Brume access token.</p><form method="post" action="/invitations/{invitation}/claim"><input type="hidden" name="return_to" value="{return_to}"><label>Existing token, optional<input name="token" autocomplete="off" placeholder="Leave empty to create one"></label><button>Accept invitation</button></form></main></body></html>"#,
        invitation = escape_html(invitation),
        return_to = escape_html(return_to),
    )
}

fn claimed_invitation_page(public_id: &str, token: Option<&str>, return_to: &str) -> String {
    let secret = token
        .map(|token| {
            format!(
                "<p><strong>Save this token now. It cannot be recovered.</strong></p><input readonly value=\"{}\" onclick=\"this.select()\">",
                escape_html(token)
            )
        })
        .unwrap_or_else(|| "<p>Your existing token now has access.</p>".to_owned());
    let destination = if return_to.is_empty() {
        String::new()
    } else {
        format!(
            "<a class=\"owner\" href=\"{}\">Open website</a>",
            escape_html(return_to)
        )
    };
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Brume access granted</title>{AUTH_STYLES}</head><body><main><div class="mark">☁️</div><h1>Access granted</h1><p>Your public recipient ID is <strong>{public_id}</strong>.</p>{secret}{destination}</main></body></html>"#,
        public_id = escape_html(public_id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_toolbar_registers_geist_outside_the_shadow_tree_and_loads_the_overlay() {
        let html = overlay_markup(Uuid::nil(), "https://auth.example.com", Some("nonce-value"));

        let style_end = html.find("</style>").unwrap();
        let font_face = html.find("@font-face").unwrap();
        assert!(font_face < style_end);
        assert!(html.contains(r#"src="/_brume/overlay.js""#));
        assert!(html.contains(r#" nonce="nonce-value""#));
        assert!(html.contains("data-brume-overlay-script"));
        assert!(html.contains(r#"data-brume-auth-origin="https://auth.example.com""#));
        assert!(html.contains("[data-brume-review-host]"));
    }

    #[test]
    fn injected_toolbar_omits_the_nonce_attribute_when_absent() {
        let html = overlay_markup(Uuid::nil(), "https://auth.example.com", None);

        assert!(!html.contains("nonce"));
        assert!(html.contains(r#"src="/_brume/overlay.js" defer"#));
    }
}
