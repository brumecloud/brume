use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use brume_core::{BASE_PATH_PLACEHOLDER, BundleManifest, validate_relative_path};
use sqlx::Row;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{access, auth::web_user, error::ApiError, state::AppState, util::random_token};

const WEB_RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.js"));
const WEB_THEME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/theme.css"));

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/_brume/runtime.js", get(runtime))
        .route("/_brume/theme.css", get(theme))
        .route("/{handle}/{slug}/_read", post(read_canonical))
        .route("/{handle}/{slug}/_assets/{*path}", get(asset_canonical))
        .route("/{handle}/{slug}", get(page_canonical_root))
        .route("/{handle}/{slug}/{*route}", get(page_canonical))
}

struct WebPlan {
    id: Uuid,
    user_id: Uuid,
    title: String,
    handle: String,
    slug: String,
    access: access::AccessControl,
    object_prefix: String,
    manifest: BundleManifest,
}

async fn page_canonical_root(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((handle, slug)): Path<(String, String)>,
) -> Response {
    serve_page(
        &state,
        &cookies,
        bearer_token(&headers),
        &handle,
        &slug,
        "/",
    )
    .await
}

async fn page_canonical(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((handle, slug, route)): Path<(String, String, String)>,
) -> Response {
    serve_page(
        &state,
        &cookies,
        bearer_token(&headers),
        &handle,
        &slug,
        &format!("/{}", route.trim_end_matches('/')),
    )
    .await
}

async fn serve_page(
    state: &AppState,
    cookies: &Cookies,
    bearer_token: Option<&str>,
    handle: &str,
    slug: &str,
    route: &str,
) -> Response {
    let result = async {
        let plan = load_plan(state, handle, slug).await?;
        let return_to = format!(
            "{}{}{}",
            state.config.plan_public_url,
            request_base(&plan),
            if route == "/" { "" } else { route }
        );
        match authorize(state, cookies, &plan, &return_to, bearer_token).await? {
            access::RequestAuthorization::Allowed => {}
            access::RequestAuthorization::Redirect(url) => {
                return Ok::<Response, ApiError>(Redirect::temporary(&url).into_response());
            }
        }
        let page = plan
            .manifest
            .pages
            .iter()
            .find(|page| page.route == route)
            .ok_or_else(ApiError::not_found)?;
        let object = state
            .storage
            .get(&format!("{}/{}", plan.object_prefix, page.object_path))
            .await
            .map_err(ApiError::internal)?;
        let fragment = std::str::from_utf8(&object.bytes)
            .map_err(ApiError::internal)?
            .replace(BASE_PATH_PLACEHOLDER, &request_base(&plan));
        let read_url = format!("{}/_read", request_base(&plan));
        let nonce = random_token("overlay_");
        let mut headers = secure_headers(Some(&nonce), Some(&state.config.auth_public_url))?;
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
        Ok((
            headers,
            Html(shell(
                &plan.title,
                &read_url,
                &fragment,
                &access::overlay_markup(
                    plan.access.id,
                    &state.config.auth_public_url,
                    Some(&nonce),
                ),
            )),
        )
            .into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn asset_canonical(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((handle, slug, path)): Path<(String, String, String)>,
) -> Response {
    serve_asset(
        &state,
        &cookies,
        bearer_token(&headers),
        &handle,
        &slug,
        &path,
    )
    .await
}

async fn serve_asset(
    state: &AppState,
    cookies: &Cookies,
    bearer_token: Option<&str>,
    handle: &str,
    slug: &str,
    path: &str,
) -> Response {
    let result = async {
        validate_relative_path(path).map_err(|_| ApiError::not_found())?;
        let plan = load_plan(state, handle, slug).await?;
        let return_to = format!("{}{}", state.config.plan_public_url, request_base(&plan));
        if !matches!(
            authorize(state, cookies, &plan, &return_to, bearer_token).await?,
            access::RequestAuthorization::Allowed
        ) {
            return Err(ApiError::not_found());
        }
        let manifest_path = format!("assets/{path}");
        let asset = plan
            .manifest
            .assets
            .iter()
            .find(|asset| asset.path == manifest_path)
            .ok_or_else(ApiError::not_found)?;
        let object = state
            .storage
            .get(&format!("{}/{}", plan.object_prefix, manifest_path))
            .await
            .map_err(ApiError::internal)?;
        let mut headers = secure_headers(None, None)?;
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&asset.content_type).map_err(ApiError::internal)?,
        );
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, max-age=86400"),
        );
        Ok::<_, ApiError>((headers, object.bytes).into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn read_canonical(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((handle, slug)): Path<(String, String)>,
) -> Response {
    record_read(&state, &cookies, bearer_token(&headers), &handle, &slug).await
}

async fn record_read(
    state: &AppState,
    cookies: &Cookies,
    bearer_token: Option<&str>,
    handle: &str,
    slug: &str,
) -> Response {
    let result = async {
        let plan = load_plan(state, handle, slug).await?;
        let return_to = format!("{}{}", state.config.plan_public_url, request_base(&plan));
        if !matches!(
            authorize(state, cookies, &plan, &return_to, bearer_token).await?,
            access::RequestAuthorization::Allowed
        ) {
            return Err(ApiError::not_found());
        }
        sqlx::query(
            "UPDATE plans SET last_read_at = now()
             WHERE id = $1
               AND (last_read_at IS NULL OR last_read_at < now() - interval '1 hour')",
        )
        .bind(plan.id)
        .execute(&state.database)
        .await?;
        Ok::<_, ApiError>(StatusCode::NO_CONTENT.into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn authorize(
    state: &AppState,
    cookies: &Cookies,
    plan: &WebPlan,
    return_to: &str,
    bearer_token: Option<&str>,
) -> Result<access::RequestAuthorization, ApiError> {
    let user = web_user(state, cookies).await?;
    access::authorize_request(
        state,
        cookies,
        &plan.access,
        return_to,
        user.is_some_and(|user| user.id == plan.user_id),
        bearer_token,
    )
    .await
}

async fn load_plan(state: &AppState, handle: &str, slug: &str) -> Result<WebPlan, ApiError> {
    let row = sqlx::query(
        "SELECT plans.id, plans.user_id, plans.title, plans.slug,
                plans.access_control_id, users.handle, plan_bundles.object_prefix,
                plan_bundles.manifest
         FROM plans
         JOIN users ON users.id = plans.user_id
         JOIN plan_bundles ON plan_bundles.id = plans.active_bundle_id
         WHERE users.handle = $1 AND plans.slug = $2 AND plans.status = 'active'",
    )
    .bind(handle)
    .bind(slug)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(ApiError::not_found)?;
    let access_control_id: Uuid = row.try_get("access_control_id")?;
    Ok(WebPlan {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        title: row.try_get("title")?,
        handle: row.try_get("handle")?,
        slug: row.try_get("slug")?,
        access: access::load_control(state, access_control_id).await?,
        object_prefix: row.try_get("object_prefix")?,
        manifest: serde_json::from_value(row.try_get("manifest")?).map_err(ApiError::internal)?,
    })
}

fn request_base(plan: &WebPlan) -> String {
    format!("/{}/{}", plan.handle, plan.slug)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

async fn runtime() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );
    (headers, WEB_RUNTIME)
}

async fn theme() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );
    (headers, WEB_THEME)
}

fn secure_headers(
    script_nonce: Option<&str>,
    overlay_origin: Option<&str>,
) -> Result<HeaderMap, ApiError> {
    let mut headers = HeaderMap::new();
    let script_source = script_nonce
        .map(|nonce| format!("'self' 'nonce-{nonce}'"))
        .unwrap_or_else(|| "'self'".to_owned());
    let frame_source = overlay_origin.unwrap_or("'none'");
    let font_source = overlay_origin.unwrap_or("'none'");
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&format!(
            "default-src 'none'; script-src {script_source}; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self'; font-src {font_source}; frame-src {frame_source}; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"
        ))
        .map_err(ApiError::internal)?,
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(headers)
}

fn shell(title: &str, read_url: &str, fragment: &str, overlay: &str) -> String {
    let title = escape_html(title);
    let read_url = escape_html(read_url);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex,nofollow\"><meta name=\"brume-read-url\" content=\"{read_url}\"><title>{title}</title><link rel=\"stylesheet\" href=\"/_brume/theme.css\"><script type=\"module\" src=\"/_brume/runtime.js\"></script></head><body><div class=\"brume-shell\">{fragment}</div>{overlay}</body></html>"
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
