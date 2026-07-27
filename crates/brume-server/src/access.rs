use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Form, Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use brume_core::{AuthMode, CreateInvitationResponse};
use chrono::{Duration, Utc};
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
const ACTION_LIFETIME: Duration = Duration::minutes(10);
const GEIST_FONT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/geist.woff2"));

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
struct SiteLocation {
    origin: String,
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

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/_brume/geist.woff2", get(geist_font))
        .route("/access", get(access_landing))
        .route("/access/token", post(submit_token))
        .route("/access/password", post(submit_password))
        .route("/invitations/{invitation}", get(invitation_landing))
        .route("/invitations/{invitation}/claim", post(claim_invitation))
        .route("/toolbar/{control_id}", get(toolbar))
        .route(
            "/toolbar/{control_id}/settings",
            post(update_toolbar_settings),
        )
        .route("/toolbar/{control_id}/invitations", post(create_invitation))
        .route("/toolbar/{control_id}/grants", post(grant_public_id))
        .route(
            "/toolbar/{control_id}/grants/revoke",
            post(revoke_public_id),
        )
}

pub fn site_router() -> Router<AppState> {
    Router::new()
        .route("/_brume/access/complete", get(complete_site_auth_handler))
        .route("/_brume/overlay-state", get(overlay_state))
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

pub fn overlay_markup(control_id: Uuid, auth_origin: &str, nonce: Option<&str>) -> String {
    let nonce = nonce
        .map(|value| format!(" nonce=\"{}\"", escape_html(value)))
        .unwrap_or_default();
    format!(
        r#"<style data-brume-overlay-style>[data-brume-overlay-host]{{all:initial;position:fixed!important;right:16px!important;bottom:16px!important;z-index:2147483647!important;color-scheme:dark!important}}[data-brume-overlay-host][data-side="left"]{{right:auto!important;left:16px!important}}</style><script{nonce} data-brume-overlay-script data-brume-site="{control_id}" data-brume-auth-origin="{auth_origin}">(() => {{
  const currentUrl = new URL(location.href);
  if (currentUrl.searchParams.get("_brume_auth_complete") === "1" && window.opener) {{
    window.opener.postMessage({{ type: "brume-owner-authenticated" }}, location.origin);
    window.close();
    return;
  }}
  const source = document.currentScript;
  if (!source || document.querySelector("[data-brume-overlay-host]")) return;
  const site = source.dataset.brumeSite;
  const authOrigin = source.dataset.brumeAuthOrigin;
  const stateUrl = "/_brume/overlay-state?site=" + encodeURIComponent(site) + "&return_to=" + encodeURIComponent(location.href);
  fetch(stateUrl, {{ credentials: "same-origin" }})
    .then((response) => response.ok ? response.json() : Promise.reject())
    .then((state) => {{
      if (!state.enabled || document.querySelector("[data-brume-overlay-host]")) return;
      const host = document.createElement("div");
      host.dataset.brumeOverlayHost = "";
      let savedSide = "right";
      try {{ savedSide = localStorage.getItem("brume-toolbar-side") === "left" ? "left" : "right"; }} catch {{}}
      host.dataset.side = savedSide;
      const root = host.attachShadow({{ mode: "closed" }});
      root.innerHTML = `<style>
        *{{box-sizing:border-box}}
        @font-face{{font-family:"Geist Variable";font-style:normal;font-weight:100 900;font-display:swap;src:url("${{authOrigin}}/_brume/geist.woff2") format("woff2")}}
        :host{{font:13px/1.35 "Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif;color:#f5f5f5}}
        button,a{{font:inherit}}
        button{{border:1px solid #3a3a3a;border-radius:9px;padding:9px 10px;background:#272727;color:#f5f5f5;cursor:pointer}}
        button:hover,a:hover{{background:#333}}
        .launcher{{width:52px;height:52px;padding:0;border:1px solid rgba(255,255,255,.15);border-radius:50%;background:rgba(20,20,20,.94);box-shadow:0 8px 28px rgba(0,0,0,.3);font-size:22px;backdrop-filter:blur(16px)}}
        .panel{{display:none;width:min(390px,calc(100vw - 32px));padding:10px;border:1px solid rgba(255,255,255,.13);border-radius:18px;background:rgba(18,18,18,.96);box-shadow:0 20px 70px rgba(0,0,0,.45);backdrop-filter:blur(22px)}}
        :host([data-open]) .panel{{display:block}}
        :host([data-open]) .launcher{{display:none}}
        header{{display:flex;align-items:center;justify-content:space-between;padding:4px 5px 10px;touch-action:none;cursor:grab}}
        header strong{{font-size:13px;font-weight:650}}
        .header-start{{display:flex;align-items:center;gap:6px}}
        .back{{display:none;width:28px;height:28px;padding:0;border:0;background:transparent;color:#aaa;font-size:18px}}
        :host([data-view="management"]) .back{{display:block}}
        .close{{width:28px;height:28px;padding:0;border:0;border-radius:8px;background:#292929;color:#aaa}}
        .actions{{display:grid;grid-template-columns:1fr 1fr;gap:7px}}
        .manage{{display:block;width:100%;margin-top:7px}}
        .result{{min-height:18px;margin:8px 4px 0;color:#9ee6bc}}
        .management{{display:none}}
        :host([data-view="management"]) .primary{{display:none}}
        :host([data-view="management"]) .management{{display:block}}
        iframe{{display:block;width:100%;height:min(520px,calc(100vh - 92px));border:0;background:transparent}}
      </style>
      <button class="launcher" type="button" aria-label="Open Brume toolbar">☁️</button>
      <div class="panel">
        <header><span class="header-start"><button class="back" type="button" aria-label="Back">‹</button><strong>Brume</strong></span><button class="close" type="button" aria-label="Close">×</button></header>
        <div class="primary">
          <div class="actions"><button class="share" type="button">Share website</button><button class="copy" type="button">Copy URL</button></div>
          <button class="manage" type="button">Manage access</button>
          <p class="result" aria-live="polite"></p>
        </div>
        <div class="management"><iframe title="Manage website access" allow="clipboard-write" sandbox="allow-forms allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"></iframe></div>
      </div>`;
      const result = root.querySelector(".result");
      const title = root.querySelector("header strong");
      const management = root.querySelector("iframe");
      const managementUrl = authOrigin + "/toolbar/" + encodeURIComponent(site) + "?return_to=" + encodeURIComponent(location.href);
      const copy = async () => {{
        await navigator.clipboard.writeText(location.href);
        result.textContent = "Copied";
      }};
      root.querySelector(".launcher").onclick = () => host.dataset.open = "";
      root.querySelector(".close").onclick = () => delete host.dataset.open;
      root.querySelector(".copy").onclick = () => copy().catch(() => result.textContent = "Could not copy");
      root.querySelector(".share").onclick = () => navigator.share
        ? navigator.share({{ url: location.href }}).catch(() => {{}})
        : copy().catch(() => result.textContent = "Could not copy");
      root.querySelector(".manage").onclick = () => {{
        if (!management.dataset.loaded) {{
          management.src = managementUrl;
          management.dataset.loaded = "true";
        }}
        host.dataset.view = "management";
        title.textContent = "Manage access";
      }};
      root.querySelector(".back").onclick = () => {{
        delete host.dataset.view;
        title.textContent = "Brume";
      }};
      addEventListener("message", (event) => {{
        if (event.origin !== location.origin || event.data?.type !== "brume-owner-authenticated") return;
        management.src = managementUrl + "&refresh=" + Date.now();
      }});
      const drag = root.querySelector("header");
      let dragging = false;
      drag.onpointerdown = (event) => {{
        if (event.target.closest("button")) return;
        dragging = true;
        drag.setPointerCapture(event.pointerId);
      }};
      drag.onpointerup = (event) => {{
        if (!dragging) return;
        dragging = false;
        const side = event.clientX < innerWidth / 2 ? "left" : "right";
        host.dataset.side = side;
        try {{ localStorage.setItem("brume-toolbar-side", side); }} catch {{}}
      }};
      document.documentElement.append(host);
    }})
    .catch(() => {{}});
}})();</script>"#,
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
}

async fn overlay_state(
    State(state): State<AppState>,
    Query(query): Query<OverlayStateQuery>,
) -> Result<Response, ApiError> {
    let control = load_control(&state, query.site).await?;
    validate_return_to(&state, control.id, &query.return_to).await?;
    let mut response = Json(OverlayState {
        enabled: control.overlay_enabled,
    })
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
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
struct ToolbarQuery {
    return_to: String,
    notice: Option<String>,
}

async fn toolbar(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Query(query): Query<ToolbarQuery>,
) -> Result<Response, ApiError> {
    let control = load_control(&state, control_id).await?;
    let location = validate_return_to(&state, control.id, &query.return_to).await?;
    let owner = auth::browser_user(&state, &cookies)
        .await?
        .filter(|user| user.id == control.owner_user_id);
    let action_token = if let Some(owner) = &owner {
        Some(create_action_token(&state, control.id, owner.id).await?)
    } else {
        None
    };
    let grants = if owner.is_some() {
        sqlx::query_scalar::<_, String>(
            "SELECT access_identities.public_id
             FROM access_grants
             JOIN access_identities ON access_identities.id = access_grants.identity_id
             WHERE access_grants.access_control_id = $1
               AND access_identities.revoked_at IS NULL
             ORDER BY access_grants.created_at",
        )
        .bind(control.id)
        .fetch_all(&state.database)
        .await?
    } else {
        Vec::new()
    };
    let html = toolbar_page(
        &control,
        &query.return_to,
        owner.is_some(),
        action_token.as_deref(),
        &grants,
        query.notice.as_deref(),
    );
    let csp = HeaderValue::from_str(&format!(
        "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; connect-src 'self'; font-src 'self'; frame-ancestors {}",
        location.origin
    ))
    .map_err(ApiError::internal)?;
    Ok((
        [
            (header::CONTENT_SECURITY_POLICY, csp),
            (
                header::REFERRER_POLICY,
                HeaderValue::from_static("no-referrer"),
            ),
            (
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ),
        ],
        Html(html),
    )
        .into_response())
}

#[derive(Deserialize)]
struct SettingsForm {
    action_token: String,
    return_to: String,
    auth: AuthMode,
    password: Option<String>,
    overlay_enabled: Option<String>,
}

async fn update_toolbar_settings(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Form(form): Form<SettingsForm>,
) -> Result<Response, ApiError> {
    let user = require_toolbar_owner(&state, &cookies, control_id, &form.action_token).await?;
    validate_return_to(&state, control_id, &form.return_to).await?;
    patch_control(
        &state,
        control_id,
        user.id,
        Some(form.auth),
        form.password.as_deref().filter(|value| !value.is_empty()),
        Some(form.overlay_enabled.is_some()),
    )
    .await?;
    Ok(toolbar_redirect(
        &state,
        control_id,
        &form.return_to,
        "Authentication updated",
    ))
}

#[derive(Deserialize)]
struct ToolbarActionForm {
    action_token: String,
    return_to: String,
    public_id: Option<String>,
}

async fn create_invitation(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Form(form): Form<ToolbarActionForm>,
) -> Result<Json<CreateInvitationResponse>, ApiError> {
    require_toolbar_owner(&state, &cookies, control_id, &form.action_token).await?;
    validate_return_to(&state, control_id, &form.return_to).await?;
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
    .bind(&form.return_to)
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

async fn grant_public_id(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Form(form): Form<ToolbarActionForm>,
) -> Result<Response, ApiError> {
    require_toolbar_owner(&state, &cookies, control_id, &form.action_token).await?;
    validate_return_to(&state, control_id, &form.return_to).await?;
    let public_id = form.public_id.unwrap_or_default();
    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM access_identities
         WHERE public_id = $1 AND revoked_at IS NULL",
    )
    .bind(public_id.trim())
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(|| ApiError::bad_request("Unknown recipient public ID"))?;
    sqlx::query(
        "INSERT INTO access_grants (access_control_id, identity_id)
         VALUES ($1, $2)
         ON CONFLICT (access_control_id, identity_id) DO NOTHING",
    )
    .bind(control_id)
    .bind(identity_id)
    .execute(&state.database)
    .await?;
    Ok(toolbar_redirect(
        &state,
        control_id,
        &form.return_to,
        "Recipient granted access",
    ))
}

async fn revoke_public_id(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(control_id): Path<Uuid>,
    Form(form): Form<ToolbarActionForm>,
) -> Result<Response, ApiError> {
    require_toolbar_owner(&state, &cookies, control_id, &form.action_token).await?;
    validate_return_to(&state, control_id, &form.return_to).await?;
    let public_id = form.public_id.unwrap_or_default();
    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM access_identities
         WHERE public_id = $1 AND revoked_at IS NULL",
    )
    .bind(public_id.trim())
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
    Ok(toolbar_redirect(
        &state,
        control_id,
        &form.return_to,
        "Recipient access revoked",
    ))
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

struct Identity {
    id: Uuid,
    public_id: String,
}

async fn identity_from_cookie(
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

async fn validate_return_to(
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

async fn create_action_token(
    state: &AppState,
    control_id: Uuid,
    owner_user_id: Uuid,
) -> Result<String, ApiError> {
    let token = random_token("action_");
    sqlx::query(
        "INSERT INTO toolbar_action_tokens (
            id, access_control_id, owner_user_id, token_hash, expires_at
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(control_id)
    .bind(owner_user_id)
    .bind(hash_secret(&token))
    .bind(Utc::now() + ACTION_LIFETIME)
    .execute(&state.database)
    .await?;
    Ok(token)
}

async fn require_toolbar_owner(
    state: &AppState,
    cookies: &Cookies,
    control_id: Uuid,
    action_token: &str,
) -> Result<auth::AuthUser, ApiError> {
    let user = auth::browser_user(state, cookies)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    let valid = sqlx::query(
        "SELECT id FROM toolbar_action_tokens
         WHERE access_control_id = $1
           AND owner_user_id = $2
           AND token_hash = $3
           AND expires_at > now()
         LIMIT 1",
    )
    .bind(control_id)
    .bind(user.id)
    .bind(hash_secret(action_token))
    .fetch_optional(&state.database)
    .await?;
    if valid.is_none() {
        return Err(ApiError::forbidden(
            "Toolbar action expired. Reopen the menu.",
        ));
    }
    let control = load_control(state, control_id).await?;
    if control.owner_user_id != user.id {
        return Err(ApiError::not_found());
    }
    Ok(user)
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

fn toolbar_redirect(state: &AppState, control_id: Uuid, return_to: &str, notice: &str) -> Response {
    Redirect::to(&format!(
        "{}/toolbar/{}?return_to={}&notice={}",
        state.config.auth_public_url,
        control_id,
        urlencoding::encode(return_to),
        urlencoding::encode(notice)
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
:root{color-scheme:light dark;font:15px/1.45 "Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif;background:#fafafa;color:#171717}
*{box-sizing:border-box}body{margin:0;min-height:100vh;display:grid;place-items:center;padding:24px}
main{width:min(100%,380px);padding:28px;border:1px solid #e7e7e7;border-radius:18px;background:rgba(255,255,255,.94);box-shadow:0 18px 55px rgba(0,0,0,.12)}
.mark{width:44px;height:44px;display:grid;place-items:center;border-radius:50%;background:#111;margin-bottom:18px}
h1{font-size:21px;margin:0 0 6px}p{margin:0 0 20px;color:#666}label{display:grid;gap:7px;font-weight:600}
input{width:100%;border:1px solid #d5d5d5;border-radius:10px;padding:11px 12px;background:transparent;color:inherit}
button{width:100%;margin-top:12px;padding:11px;border:0;border-radius:10px;background:#171717;color:white;font-weight:650;cursor:pointer}
.owner{display:block;margin-top:18px;text-align:center;color:#666;font-size:13px}.error{color:#b42318;background:#fee4e2;padding:9px 11px;border-radius:9px}
@media(prefers-color-scheme:dark){:root{background:#0a0a0a;color:#ededed}main{background:rgba(20,20,20,.96);border-color:#333}p,.owner{color:#aaa}input{border-color:#444}}
</style>"#;

fn toolbar_page(
    control: &AccessControl,
    share_url: &str,
    owner: bool,
    action_token: Option<&str>,
    grants: &[String],
    notice: Option<&str>,
) -> String {
    let owner_controls = if owner {
        let token = escape_html(action_token.unwrap_or_default());
        let selected = |mode| {
            if control.auth_mode == mode {
                "selected"
            } else {
                ""
            }
        };
        let grant_rows = grants
            .iter()
            .map(|public_id| {
                format!(
                    r#"<form class="grant" method="post" action="/toolbar/{id}/grants/revoke"><input type="hidden" name="action_token" value="{token}"><input type="hidden" name="return_to" value="{return_to}"><input type="hidden" name="public_id" value="{public_id}"><code>{public_id}</code><button type="submit">Revoke</button></form>"#,
                    id = control.id,
                    return_to = escape_html(share_url),
                    public_id = escape_html(public_id),
                )
            })
            .collect::<String>();
        let token_sharing = if control.auth_mode == AuthMode::Token {
            format!(
                r#"<section><h2>Token sharing</h2><button type="button" data-invite data-action="/toolbar/{id}/invitations" data-token="{token}">Create one-day invite</button><form method="post" action="/toolbar/{id}/grants"><input type="hidden" name="action_token" value="{token}"><input type="hidden" name="return_to" value="{return_to}"><input name="public_id" placeholder="Recipient public ID" required><button type="submit">Grant access</button></form>{grant_rows}</section>"#,
                id = control.id,
                return_to = escape_html(share_url),
            )
        } else {
            String::new()
        };
        format!(
            r#"<section><h2>Authentication</h2><form method="post" action="/toolbar/{id}/settings"><input type="hidden" name="action_token" value="{token}"><input type="hidden" name="return_to" value="{return_to}"><select name="auth" data-auth-select><option value="token" {token_selected}>Token</option><option value="password" {password_selected}>Password</option><option value="none" {none_selected}>None</option></select><input name="password" data-password type="password" minlength="8" placeholder="New password when needed"><label class="check"><input type="checkbox" name="overlay_enabled" {checked}>Show cloud toolbar</label><button type="submit">Save</button></form></section>{token_sharing}"#,
            id = control.id,
            return_to = escape_html(share_url),
            token_selected = selected(AuthMode::Token),
            password_selected = selected(AuthMode::Password),
            none_selected = selected(AuthMode::None),
            checked = if control.overlay_enabled {
                "checked"
            } else {
                ""
            },
        )
    } else {
        let popup_return_to = Url::parse(share_url)
            .map(|mut url| {
                url.query_pairs_mut()
                    .append_pair("_brume_auth_complete", "1");
                url.to_string()
            })
            .unwrap_or_else(|_| share_url.to_owned());
        format!(
            "<p class=\"muted\">Only the person who deployed this website can change access.</p><a class=\"signin\" target=\"brume-auth\" href=\"/auth/github/start?site={}&site_return_to={}\">Sign in to manage</a>",
            control.id,
            urlencoding::encode(&popup_return_to)
        )
    };
    let notice = notice
        .map(|value| format!("<p class=\"notice\">{}</p>", escape_html(value)))
        .unwrap_or_default();
    let share_json = serde_json::to_string(share_url).unwrap_or_else(|_| "\"\"".to_owned());
    format!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Brume toolbar</title><style>
@font-face{{font-family:"Geist Variable";font-style:normal;font-weight:100 900;font-display:swap;src:url("/_brume/geist.woff2") format("woff2")}}
*{{box-sizing:border-box}}:root{{color-scheme:dark;font:13px/1.35 "Geist Variable",Geist,ui-sans-serif,system-ui,sans-serif;color:#f5f5f5}}
body{{margin:0;background:transparent;overflow:hidden}}button,input,select{{font:inherit}}
.launcher{{position:absolute;right:0;bottom:0;width:52px;height:52px;border:1px solid rgba(255,255,255,.15);border-radius:50%;background:rgba(20,20,20,.94);box-shadow:0 8px 28px rgba(0,0,0,.3);cursor:pointer;font-size:22px;backdrop-filter:blur(16px)}}
.panel{{display:none;position:absolute;right:0;bottom:0;width:370px;max-height:465px;overflow:auto;padding:10px;border:1px solid rgba(255,255,255,.13);border-radius:18px;background:rgba(18,18,18,.96);box-shadow:0 20px 70px rgba(0,0,0,.45);backdrop-filter:blur(22px)}}
.open .panel{{display:block}}.open .launcher{{display:none}}header{{display:flex;align-items:center;justify-content:space-between;padding:4px 5px 10px}}header strong{{font-size:13px}}.close{{width:28px;height:28px;border:0;border-radius:8px;background:#292929;color:#aaa;cursor:pointer}}
.actions{{display:grid;grid-template-columns:1fr 1fr;gap:7px}}button{{border:1px solid #3a3a3a;border-radius:9px;padding:9px 10px;background:#272727;color:#f5f5f5;cursor:pointer}}button:hover{{background:#333}}
section{{border-top:1px solid #303030;margin-top:10px;padding:10px 4px 2px}}h2{{font-size:11px;text-transform:uppercase;letter-spacing:.08em;color:#888;margin:0 0 8px}}
form{{display:grid;gap:7px;margin-top:7px}}input,select{{width:100%;border:1px solid #3a3a3a;border-radius:8px;background:#202020;color:#eee;padding:8px}}.check{{display:flex;gap:7px;align-items:center;color:#bbb}}.check input{{width:auto}}
.muted{{color:#9a9a9a;margin:12px 5px}}.notice{{background:#173c2b;color:#9ee6bc;border-radius:8px;padding:8px}}.result{{color:#9ee6bc;word-break:break-all}}
.signin{{display:block;margin:8px 5px;color:#ddd}}
.grant{{display:grid;grid-template-columns:1fr auto;align-items:center}}.grant code{{overflow:hidden;text-overflow:ellipsis;color:#aaa}}.grant button{{padding:6px 8px}}
body.embedded{{min-height:0;overflow:auto}}body.embedded .launcher,body.embedded header,body.embedded .actions{{display:none}}
body.embedded .panel{{display:block;position:static;width:100%;max-height:none;overflow:visible;padding:0;border:0;border-radius:0;background:transparent;box-shadow:none;backdrop-filter:none}}
body.embedded section:first-of-type{{margin-top:0;border-top:0;padding-top:2px}}
@media(prefers-reduced-motion:no-preference){{.panel,.launcher{{transition:opacity .14s ease,transform .14s ease}}}}
</style></head><body><div id="toolbar"><button class="launcher" aria-label="Open Brume toolbar">☁️</button><div class="panel"><header><strong>Brume</strong><button class="close" aria-label="Close">×</button></header>{notice}<div class="actions"><button type="button" data-share>Share website</button><button type="button" data-copy>Copy URL</button></div>{owner_controls}<p class="result" aria-live="polite"></p></div></div><script>
const root=document.querySelector("#toolbar");const result=document.querySelector(".result");const shareUrl={share_json};
if(window.self!==window.top)document.body.classList.add("embedded");
function setOpen(open){{root.classList.toggle("open",open)}}
document.querySelector(".launcher").onclick=()=>setOpen(true);document.querySelector(".close").onclick=()=>setOpen(false);setOpen(true);
async function copy(value){{await navigator.clipboard.writeText(value);result.textContent="Copied"}}
document.querySelector("[data-copy]").onclick=()=>copy(shareUrl);
document.querySelector("[data-share]").onclick=()=>navigator.share?navigator.share({{url:shareUrl}}):copy(shareUrl);
const invite=document.querySelector("[data-invite]");if(invite)invite.onclick=async()=>{{const body=new URLSearchParams({{action_token:invite.dataset.token,return_to:shareUrl}});const response=await fetch(invite.dataset.action,{{method:"POST",body}});const data=await response.json();if(response.ok){{await copy(data.url);result.textContent="Invite copied. It expires in one day."}}else result.textContent=data.message??"Could not create invite"}};
const authSelect=document.querySelector("[data-auth-select]");const password=document.querySelector("[data-password]");if(authSelect&&password){{const updatePassword=()=>password.required=authSelect.value==="password"&&{password_required};authSelect.onchange=updatePassword;updatePassword()}}
</script></body></html>"##,
        password_required = control.auth_mode != AuthMode::Password,
    )
}

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
