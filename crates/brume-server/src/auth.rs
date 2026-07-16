use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, header, request::Parts},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use brume_core::{BeginCliLoginResponse, PollCliLoginResponse};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::Row;
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

const WEB_SESSION_COOKIE: &str = "brume_session";

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    #[allow(dead_code)]
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
        let token_hash = hash_secret(authorization);
        let row = sqlx::query(
            "SELECT users.id, users.handle
             FROM api_tokens
             JOIN users ON users.id = api_tokens.user_id
             WHERE api_tokens.token_hash = $1
               AND api_tokens.revoked_at IS NULL",
        )
        .bind(token_hash)
        .fetch_optional(&state.database)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
        let user = Self {
            id: row.try_get("id")?,
            handle: row.try_get("handle")?,
        };
        let _ = sqlx::query(
            "UPDATE api_tokens SET last_used_at = now()
             WHERE token_hash = $1
               AND (last_used_at IS NULL OR last_used_at < now() - interval '1 hour')",
        )
        .bind(hash_secret(authorization))
        .execute(&state.database)
        .await;
        Ok(user)
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/auth/cli/sessions", post(begin_cli_login))
        .route(
            "/api/v1/auth/cli/sessions/{session_id}/poll",
            post(poll_cli_login),
        )
        .route("/auth/github/start", get(github_start))
        .route("/auth/github/callback", get(github_callback))
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
            state.config.public_url
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
        "SELECT poll_secret_hash, issued_token, expires_at, consumed_at, users.handle
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
    let expires_at: chrono::DateTime<Utc> = row.try_get("expires_at")?;
    if expires_at <= Utc::now()
        || row
            .try_get::<Option<chrono::DateTime<Utc>>, _>("consumed_at")?
            .is_some()
    {
        transaction.commit().await?;
        return Ok(Json(PollCliLoginResponse::Expired));
    }
    let issued_token: Option<String> = row.try_get("issued_token")?;
    let Some(token) = issued_token else {
        transaction.commit().await?;
        return Ok(Json(PollCliLoginResponse::Pending));
    };
    let user_handle: String = row.try_get("handle")?;
    sqlx::query(
        "UPDATE cli_login_sessions
         SET consumed_at = now(), issued_token = NULL
         WHERE id = $1",
    )
    .bind(session_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Json(PollCliLoginResponse::Authorized {
        token,
        user_handle,
    }))
}

#[derive(Deserialize)]
struct GithubStartQuery {
    cli_session: Option<Uuid>,
    return_to: Option<String>,
}

async fn github_start(
    State(state): State<AppState>,
    Query(query): Query<GithubStartQuery>,
) -> Result<Redirect, ApiError> {
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
    let return_to = query
        .return_to
        .filter(|value| value.starts_with('/') && !value.starts_with("//"));
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
            &format!("{}/auth/github/callback", state.config.public_url),
        )
        .append_pair("scope", "read:user")
        .append_pair("state", &oauth_state);
    Ok(Redirect::temporary(authorize.as_str()))
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
        let api_token = issue_api_token(&state, user.id).await?;
        sqlx::query(
            "UPDATE cli_login_sessions
             SET user_id = $1, issued_token = $2, authorized_at = now()
             WHERE id = $3 AND expires_at > now() AND consumed_at IS NULL",
        )
        .bind(user.id)
        .bind(api_token)
        .bind(cli_session_id)
        .execute(&state.database)
        .await?;
    }
    let web_token = random_token("web_");
    sqlx::query(
        "INSERT INTO web_sessions (id, user_id, session_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(user.id)
    .bind(hash_secret(&web_token))
    .bind(Utc::now() + Duration::days(30))
    .execute(&state.database)
    .await?;
    let cookie = Cookie::build((WEB_SESSION_COOKIE, web_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.config.public_url.starts_with("https://"))
        .max_age(time::Duration::days(30))
        .build();
    cookies.add(cookie);
    let return_to: Option<String> = oauth.try_get("return_to")?;
    if let Some(return_to) = return_to {
        Ok(Redirect::to(&return_to).into_response())
    } else {
        Ok(Html(
            "<!doctype html><html><body><h1>Brume login complete</h1><p>You can close this tab and return to the CLI.</p></body></html>",
        )
        .into_response())
    }
}

pub async fn web_user(state: &AppState, cookies: &Cookies) -> Result<Option<AuthUser>, ApiError> {
    let Some(cookie) = cookies.get(WEB_SESSION_COOKIE) else {
        return Ok(None);
    };
    let row = sqlx::query(
        "SELECT users.id, users.handle
         FROM web_sessions
         JOIN users ON users.id = web_sessions.user_id
         WHERE web_sessions.session_hash = $1 AND web_sessions.expires_at > now()",
    )
    .bind(hash_secret(cookie.value()))
    .fetch_optional(&state.database)
    .await?;
    row.map(|row| {
        Ok(AuthUser {
            id: row.try_get("id")?,
            handle: row.try_get("handle")?,
        })
    })
    .transpose()
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

pub async fn issue_api_token(state: &AppState, user_id: Uuid) -> Result<String, ApiError> {
    let token = random_token("brume_");
    sqlx::query(
        "INSERT INTO api_tokens (id, user_id, token_hash)
         VALUES ($1, $2, $3)",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(hash_secret(&token))
    .execute(&state.database)
    .await?;
    Ok(token)
}
