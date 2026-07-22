use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, header, request::Parts},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use brume_core::{BeginCliLoginResponse, PollCliLoginResponse, RefreshTokenRequest, TokenPair};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::{Postgres, Row, Transaction};
use tower_cookies::{
    Cookies,
    cookie::{Cookie, SameSite, time},
};
use url::Url;
use uuid::Uuid;

use crate::{
    error::ApiError,
    state::AppState,
    util::{github_handle, hash_secret, random_token},
};

const PLAN_ACCESS_COOKIE: &str = "brume_plan_access";
const AUTH_REFRESH_COOKIE: &str = "brume_auth_refresh";
const ACCESS_LIFETIME: Duration = Duration::hours(1);
const REFRESH_LIFETIME: Duration = Duration::days(90);
const TICKET_LIFETIME: Duration = Duration::minutes(1);

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub handle: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(ApiError::unauthorized)?;
        authenticate_access_token(state, authorization).await
    }
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/auth/cli/sessions", post(begin_cli_login))
        .route(
            "/api/v1/auth/cli/sessions/{session_id}/poll",
            post(poll_cli_login),
        )
        .route("/api/v1/auth/tokens/refresh", post(refresh_cli_token))
}

pub fn browser_router() -> Router<AppState> {
    Router::new()
        .route("/auth/github/start", get(github_start))
        .route("/auth/github/callback", get(github_callback))
}

pub fn plan_router() -> Router<AppState> {
    Router::new().route("/_brume/auth/complete", get(complete_plan_auth))
}

async fn authenticate_access_token(state: &AppState, token: &str) -> Result<AuthUser, ApiError> {
    let token_hash = hash_secret(token);
    let row = sqlx::query(
        "SELECT users.id, users.handle
         FROM access_tokens
         JOIN token_families ON token_families.id = access_tokens.family_id
         JOIN users ON users.id = access_tokens.user_id
         WHERE access_tokens.token_hash = $1
           AND access_tokens.expires_at > now()
           AND access_tokens.revoked_at IS NULL
           AND token_families.expires_at > now()
           AND token_families.revoked_at IS NULL",
    )
    .bind(&token_hash)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(ApiError::unauthorized)?;
    let _ = sqlx::query(
        "UPDATE access_tokens SET last_used_at = now()
         WHERE token_hash = $1
           AND (last_used_at IS NULL OR last_used_at < now() - interval '1 hour')",
    )
    .bind(token_hash)
    .execute(&state.database)
    .await;
    Ok(AuthUser {
        id: row.try_get("id")?,
        handle: row.try_get("handle")?,
    })
}

async fn begin_cli_login(
    State(state): State<AppState>,
) -> Result<Json<BeginCliLoginResponse>, ApiError> {
    let session_id = Uuid::now_v7();
    let poll_secret = random_token("poll_");
    sqlx::query(
        "INSERT INTO cli_login_sessions (id, poll_secret_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(session_id)
    .bind(hash_secret(&poll_secret))
    .bind(Utc::now() + Duration::minutes(15))
    .execute(&state.database)
    .await?;
    Ok(Json(BeginCliLoginResponse {
        session_id,
        browser_url: format!(
            "{}/auth/github/start?cli_session={session_id}",
            state.config.auth_public_url
        ),
        poll_secret,
        expires_in_seconds: 900,
    }))
}

async fn poll_cli_login(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<PollCliLoginResponse>, ApiError> {
    let poll_secret = headers
        .get("x-brume-poll-secret")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::unauthorized)?;
    let mut transaction = state.database.begin().await?;
    let row = sqlx::query(
        "SELECT cli_login_sessions.poll_secret_hash,
                cli_login_sessions.expires_at,
                cli_login_sessions.consumed_at,
                cli_login_sessions.user_id,
                users.handle
         FROM cli_login_sessions
         LEFT JOIN users ON users.id = cli_login_sessions.user_id
         WHERE cli_login_sessions.id = $1
         FOR UPDATE OF cli_login_sessions",
    )
    .bind(session_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(ApiError::unauthorized)?;
    let expected: Vec<u8> = row.try_get("poll_secret_hash")?;
    if expected != hash_secret(poll_secret) {
        return Err(ApiError::unauthorized());
    }
    let expires_at: DateTime<Utc> = row.try_get("expires_at")?;
    if expires_at <= Utc::now()
        || row
            .try_get::<Option<DateTime<Utc>>, _>("consumed_at")?
            .is_some()
    {
        transaction.commit().await?;
        return Ok(Json(PollCliLoginResponse::Expired));
    }
    let Some(user_id) = row.try_get::<Option<Uuid>, _>("user_id")? else {
        transaction.commit().await?;
        return Ok(Json(PollCliLoginResponse::Pending));
    };
    let credentials = issue_new_family(&mut transaction, user_id, "cli").await?;
    let user_handle: String = row.try_get("handle")?;
    sqlx::query("UPDATE cli_login_sessions SET consumed_at = now() WHERE id = $1")
        .bind(session_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Json(PollCliLoginResponse::Authorized {
        credentials,
        user_handle,
    }))
}

async fn refresh_cli_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<TokenPair>, ApiError> {
    Ok(Json(
        rotate_refresh_token(&state, &request.refresh_token)
            .await?
            .pair,
    ))
}

#[derive(Deserialize)]
struct GithubStartQuery {
    cli_session: Option<Uuid>,
    return_to: Option<String>,
}

async fn github_start(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<GithubStartQuery>,
) -> Result<Response, ApiError> {
    if let Some(session_id) = query.cli_session {
        let valid: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM cli_login_sessions
                WHERE id = $1 AND expires_at > now() AND consumed_at IS NULL
            )",
        )
        .bind(session_id)
        .fetch_one(&state.database)
        .await?;
        if !valid {
            return Err(ApiError::bad_request(
                "CLI login session is invalid or expired",
            ));
        }
    }
    let return_to = sanitize_return_to(query.return_to);
    if query.cli_session.is_none()
        && let (Some(return_to), Some(refresh)) =
            (return_to.as_deref(), cookies.get(AUTH_REFRESH_COOKIE))
    {
        match rotate_refresh_token(&state, refresh.value()).await {
            Ok(rotated) => {
                set_refresh_cookie(&state, &cookies, &rotated.pair);
                return Ok(create_auth_ticket(
                    &state,
                    rotated.user_id,
                    rotated.family_id,
                    return_to,
                )
                .await?
                .into_response());
            }
            Err(_) => remove_refresh_cookie(&cookies),
        }
    }
    let oauth_state = random_token("state_");
    sqlx::query(
        "INSERT INTO oauth_states (id, state_hash, cli_session_id, return_to, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(hash_secret(&oauth_state))
    .bind(query.cli_session)
    .bind(return_to)
    .bind(Utc::now() + Duration::minutes(10))
    .execute(&state.database)
    .await?;
    let mut authorize =
        Url::parse("https://github.com/login/oauth/authorize").map_err(ApiError::internal)?;
    authorize
        .query_pairs_mut()
        .append_pair("client_id", &state.config.github_client_id)
        .append_pair(
            "redirect_uri",
            &format!("{}/auth/github/callback", state.config.auth_public_url),
        )
        .append_pair("scope", "read:user")
        .append_pair("state", &oauth_state);
    Ok(Redirect::temporary(authorize.as_str()).into_response())
}

#[derive(Deserialize)]
struct GithubCallbackQuery {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
}

async fn github_callback(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<GithubCallbackQuery>,
) -> Result<Response, ApiError> {
    let oauth = sqlx::query(
        "DELETE FROM oauth_states
         WHERE state_hash = $1 AND expires_at > now()
         RETURNING cli_session_id, return_to",
    )
    .bind(hash_secret(&query.state))
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(|| ApiError::bad_request("OAuth state is invalid or expired"))?;
    let token_response = state
        .http
        .post("https://github.com/login/oauth/access_token")
        .header(header::ACCEPT, "application/json")
        .form(&HashMap::from([
            ("client_id", state.config.github_client_id.as_str()),
            ("client_secret", state.config.github_client_secret.as_str()),
            ("code", query.code.as_str()),
        ]))
        .send()
        .await
        .map_err(ApiError::internal)?
        .json::<GithubTokenResponse>()
        .await
        .map_err(ApiError::internal)?;
    let github_token = token_response.access_token.ok_or_else(|| {
        ApiError::forbidden(
            token_response
                .error_description
                .unwrap_or_else(|| "GitHub did not authorize Brume".to_owned()),
        )
    })?;
    let github_user = state
        .http
        .get("https://api.github.com/user")
        .bearer_auth(github_token)
        .header(header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(ApiError::internal)?
        .error_for_status()
        .map_err(ApiError::internal)?
        .json::<GithubUser>()
        .await
        .map_err(ApiError::internal)?;
    if !state.config.github_allowed_ids.is_empty()
        && !state.config.github_allowed_ids.contains(&github_user.id)
    {
        return Err(ApiError::forbidden(
            "This GitHub account is not allowed to use Brume",
        ));
    }

    let user = provision_user(&state, github_user.id, &github_user.login).await?;
    let cli_session_id: Option<Uuid> = oauth.try_get("cli_session_id")?;
    if let Some(cli_session_id) = cli_session_id {
        sqlx::query(
            "UPDATE cli_login_sessions
             SET user_id = $1, authorized_at = now()
             WHERE id = $2 AND expires_at > now() AND consumed_at IS NULL",
        )
        .bind(user.id)
        .bind(cli_session_id)
        .execute(&state.database)
        .await?;
    }

    let mut transaction = state.database.begin().await?;
    let browser = issue_new_family_with_id(&mut transaction, user.id, "browser").await?;
    transaction.commit().await?;
    set_refresh_cookie(&state, &cookies, &browser.pair);

    let return_to: Option<String> = oauth.try_get("return_to")?;
    if let Some(return_to) = return_to {
        Ok(
            create_auth_ticket(&state, user.id, browser.family_id, &return_to)
                .await?
                .into_response(),
        )
    } else {
        Ok(Html(
            "<!doctype html><html><body><h1>Brume login complete</h1><p>You can close this tab and return to the CLI.</p></body></html>",
        )
        .into_response())
    }
}

#[derive(Deserialize)]
struct TicketQuery {
    ticket: String,
}

async fn complete_plan_auth(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<TicketQuery>,
) -> Result<Response, ApiError> {
    let mut transaction = state.database.begin().await?;
    let row = sqlx::query(
        "UPDATE auth_tickets
         SET consumed_at = now()
         WHERE ticket_hash = $1 AND expires_at > now() AND consumed_at IS NULL
         RETURNING user_id, family_id, return_to",
    )
    .bind(hash_secret(&query.ticket))
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::bad_request("Authentication ticket is invalid or expired"))?;
    let user_id: Uuid = row.try_get("user_id")?;
    let family_id: Uuid = row.try_get("family_id")?;
    let return_to: String = row.try_get("return_to")?;
    let (access_token, access_expires_at) =
        issue_access_token(&mut transaction, user_id, family_id).await?;
    transaction.commit().await?;
    set_plan_access_cookie(&state, &cookies, &access_token, access_expires_at);
    Ok(Redirect::to(&return_to).into_response())
}

async fn create_auth_ticket(
    state: &AppState,
    user_id: Uuid,
    family_id: Uuid,
    return_to: &str,
) -> Result<Redirect, ApiError> {
    let ticket = random_token("ticket_");
    sqlx::query(
        "INSERT INTO auth_tickets (
            id, user_id, family_id, ticket_hash, return_to, expires_at
         ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(family_id)
    .bind(hash_secret(&ticket))
    .bind(return_to)
    .bind(Utc::now() + TICKET_LIFETIME)
    .execute(&state.database)
    .await?;
    Ok(Redirect::to(&format!(
        "{}/_brume/auth/complete?ticket={}",
        state.config.plan_public_url,
        urlencoding::encode(&ticket)
    )))
}

fn sanitize_return_to(value: Option<String>) -> Option<String> {
    value.filter(|value| value.starts_with('/') && !value.starts_with("//"))
}

fn set_refresh_cookie(state: &AppState, cookies: &Cookies, pair: &TokenPair) {
    let max_age = (pair.refresh_expires_at - Utc::now()).num_seconds().max(0);
    cookies.add(
        Cookie::build((AUTH_REFRESH_COOKIE, pair.refresh_token.clone()))
            .path("/auth")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(state.config.auth_public_url.starts_with("https://"))
            .max_age(time::Duration::seconds(max_age))
            .build(),
    );
}

fn remove_refresh_cookie(cookies: &Cookies) {
    cookies.remove(Cookie::build(AUTH_REFRESH_COOKIE).path("/auth").build());
}

fn set_plan_access_cookie(
    state: &AppState,
    cookies: &Cookies,
    token: &str,
    expires_at: DateTime<Utc>,
) {
    let max_age = (expires_at - Utc::now()).num_seconds().max(0);
    cookies.add(
        Cookie::build((PLAN_ACCESS_COOKIE, token.to_owned()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(state.config.plan_public_url.starts_with("https://"))
            .max_age(time::Duration::seconds(max_age))
            .build(),
    );
}

pub async fn web_user(state: &AppState, cookies: &Cookies) -> Result<Option<AuthUser>, ApiError> {
    let Some(cookie) = cookies.get(PLAN_ACCESS_COOKIE) else {
        return Ok(None);
    };
    match authenticate_access_token(state, cookie.value()).await {
        Ok(user) => Ok(Some(user)),
        Err(error) if error.status() == axum::http::StatusCode::UNAUTHORIZED => Ok(None),
        Err(error) => Err(error),
    }
}

struct IssuedFamily {
    family_id: Uuid,
    pair: TokenPair,
}

struct RotatedFamily {
    user_id: Uuid,
    family_id: Uuid,
    pair: TokenPair,
}

async fn issue_new_family(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    client_type: &str,
) -> Result<TokenPair, ApiError> {
    Ok(issue_new_family_with_id(transaction, user_id, client_type)
        .await?
        .pair)
}

async fn issue_new_family_with_id(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    client_type: &str,
) -> Result<IssuedFamily, ApiError> {
    let family_id = Uuid::now_v7();
    let refresh_expires_at = Utc::now() + REFRESH_LIFETIME;
    sqlx::query(
        "INSERT INTO token_families (id, user_id, client_type, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(family_id)
    .bind(user_id)
    .bind(client_type)
    .bind(refresh_expires_at)
    .execute(&mut **transaction)
    .await?;
    let pair = issue_pair(transaction, user_id, family_id, refresh_expires_at)
        .await?
        .pair;
    Ok(IssuedFamily { family_id, pair })
}

struct IssuedPair {
    refresh_id: Uuid,
    pair: TokenPair,
}

async fn issue_pair(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    family_id: Uuid,
    refresh_expires_at: DateTime<Utc>,
) -> Result<IssuedPair, ApiError> {
    let (access_token, access_expires_at) =
        issue_access_token(transaction, user_id, family_id).await?;
    let refresh_token = random_token("refresh_");
    let refresh_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, family_id, token_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(refresh_id)
    .bind(family_id)
    .bind(hash_secret(&refresh_token))
    .bind(refresh_expires_at)
    .execute(&mut **transaction)
    .await?;
    Ok(IssuedPair {
        refresh_id,
        pair: TokenPair {
            access_token,
            refresh_token,
            access_expires_at,
            refresh_expires_at,
        },
    })
}

async fn issue_access_token(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    family_id: Uuid,
) -> Result<(String, DateTime<Utc>), ApiError> {
    let token = random_token("access_");
    let expires_at = Utc::now() + ACCESS_LIFETIME;
    sqlx::query(
        "INSERT INTO access_tokens (id, family_id, user_id, token_hash, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(family_id)
    .bind(user_id)
    .bind(hash_secret(&token))
    .bind(expires_at)
    .execute(&mut **transaction)
    .await?;
    Ok((token, expires_at))
}

async fn rotate_refresh_token(
    state: &AppState,
    refresh_token: &str,
) -> Result<RotatedFamily, ApiError> {
    let mut transaction = state.database.begin().await?;
    let row = sqlx::query(
        "SELECT refresh_tokens.id, refresh_tokens.family_id,
                refresh_tokens.expires_at, refresh_tokens.consumed_at,
                token_families.user_id, token_families.expires_at AS family_expires_at,
                token_families.revoked_at
         FROM refresh_tokens
         JOIN token_families ON token_families.id = refresh_tokens.family_id
         WHERE refresh_tokens.token_hash = $1
         FOR UPDATE OF refresh_tokens, token_families",
    )
    .bind(hash_secret(refresh_token))
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(ApiError::unauthorized)?;
    let family_id: Uuid = row.try_get("family_id")?;
    if row
        .try_get::<Option<DateTime<Utc>>, _>("consumed_at")?
        .is_some()
    {
        sqlx::query("UPDATE token_families SET revoked_at = now() WHERE id = $1")
            .bind(family_id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        return Err(ApiError::unauthorized());
    }
    let expires_at: DateTime<Utc> = row.try_get("expires_at")?;
    let family_expires_at: DateTime<Utc> = row.try_get("family_expires_at")?;
    let revoked_at: Option<DateTime<Utc>> = row.try_get("revoked_at")?;
    if expires_at <= Utc::now() || family_expires_at <= Utc::now() || revoked_at.is_some() {
        transaction.commit().await?;
        return Err(ApiError::unauthorized());
    }
    let user_id: Uuid = row.try_get("user_id")?;
    let old_refresh_id: Uuid = row.try_get("id")?;
    let issued = issue_pair(&mut transaction, user_id, family_id, family_expires_at).await?;
    sqlx::query(
        "UPDATE refresh_tokens
         SET consumed_at = now(), replaced_by = $1
         WHERE id = $2",
    )
    .bind(issued.refresh_id)
    .bind(old_refresh_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(RotatedFamily {
        user_id,
        family_id,
        pair: issued.pair,
    })
}

pub async fn provision_user(
    state: &AppState,
    github_id: i64,
    login: &str,
) -> Result<AuthUser, ApiError> {
    let handle = github_handle(login);
    let row = sqlx::query(
        "INSERT INTO users (id, github_id, github_login, handle)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (github_id) DO UPDATE
         SET github_login = EXCLUDED.github_login, updated_at = now()
         RETURNING id, handle",
    )
    .bind(Uuid::now_v7())
    .bind(github_id)
    .bind(login)
    .bind(handle)
    .fetch_one(&state.database)
    .await?;
    Ok(AuthUser {
        id: row.try_get("id")?,
        handle: row.try_get("handle")?,
    })
}

pub async fn issue_development_credentials(
    state: &AppState,
    user_id: Uuid,
) -> Result<TokenPair, ApiError> {
    let mut transaction = state.database.begin().await?;
    let credentials = issue_new_family(&mut transaction, user_id, "development").await?;
    transaction.commit().await?;
    Ok(credentials)
}
