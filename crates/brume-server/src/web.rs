use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use brume_core::{BASE_PATH_PLACEHOLDER, BundleManifest, Visibility, validate_relative_path};
use sqlx::Row;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{auth::web_user, error::ApiError, state::AppState, util::hash_secret};

const WEB_RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.js"));
const WEB_THEME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/theme.css"));

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/_brume/runtime.js", get(runtime))
        .route("/_brume/theme.css", get(theme))
        .route("/@{handle}/{slug}/_read", post(read_canonical))
        .route("/@{handle}/{slug}/_assets/{*path}", get(asset_canonical))
        .route("/@{handle}/{slug}/~{access}/_read", post(read_shared))
        .route(
            "/@{handle}/{slug}/~{access}/_assets/{*path}",
            get(asset_shared),
        )
        .route("/@{handle}/{slug}", get(page_canonical_root))
        .route("/@{handle}/{slug}/~{access}", get(page_shared_root))
        .route("/@{handle}/{slug}/~{access}/{*route}", get(page_shared))
        .route("/@{handle}/{slug}/{*route}", get(page_canonical))
}

struct WebPlan {
    id: Uuid,
    user_id: Uuid,
    title: String,
    handle: String,
    slug: String,
    visibility: Visibility,
    unlisted_token_hash: Option<Vec<u8>>,
    object_prefix: String,
    manifest: BundleManifest,
}

enum Access<'a> {
    Canonical,
    Shared(&'a str),
}

async fn page_canonical_root(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug)): Path<(String, String)>,
) -> Response {
    serve_page(&state, &cookies, &handle, &slug, Access::Canonical, "/").await
}

async fn page_canonical(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug, route)): Path<(String, String, String)>,
) -> Response {
    serve_page(
        &state,
        &cookies,
        &handle,
        &slug,
        Access::Canonical,
        &format!("/{}", route.trim_end_matches('/')),
    )
    .await
}

async fn page_shared_root(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug, access)): Path<(String, String, String)>,
) -> Response {
    serve_page(
        &state,
        &cookies,
        &handle,
        &slug,
        Access::Shared(&access),
        "/",
    )
    .await
}

async fn page_shared(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug, access, route)): Path<(String, String, String, String)>,
) -> Response {
    serve_page(
        &state,
        &cookies,
        &handle,
        &slug,
        Access::Shared(&access),
        &format!("/{}", route.trim_end_matches('/')),
    )
    .await
}

async fn serve_page(
    state: &AppState,
    cookies: &Cookies,
    handle: &str,
    slug: &str,
    access: Access<'_>,
    route: &str,
) -> Response {
    let result = async {
        let plan = load_plan(state, handle, slug).await?;
        match authorize(state, cookies, &plan, &access).await? {
            Authorization::Allowed => {}
            Authorization::LoginRequired => {
                let return_to = request_base(&plan, &access);
                return Ok::<Response, ApiError>(
                    Redirect::temporary(&format!(
                        "/auth/github/start?return_to={}",
                        urlencoding::encode(&return_to)
                    ))
                    .into_response(),
                );
            }
            Authorization::Hidden => return Err(ApiError::not_found()),
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
            .replace(BASE_PATH_PLACEHOLDER, &request_base(&plan, &access));
        let read_url = format!("{}/_read", request_base(&plan, &access));
        let mut headers = secure_headers();
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
        Ok((headers, Html(shell(&plan.title, &read_url, &fragment))).into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn asset_canonical(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug, path)): Path<(String, String, String)>,
) -> Response {
    serve_asset(&state, &cookies, &handle, &slug, Access::Canonical, &path).await
}

async fn asset_shared(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug, access, path)): Path<(String, String, String, String)>,
) -> Response {
    serve_asset(
        &state,
        &cookies,
        &handle,
        &slug,
        Access::Shared(&access),
        &path,
    )
    .await
}

async fn serve_asset(
    state: &AppState,
    cookies: &Cookies,
    handle: &str,
    slug: &str,
    access: Access<'_>,
    path: &str,
) -> Response {
    let result = async {
        validate_relative_path(path).map_err(|_| ApiError::not_found())?;
        let plan = load_plan(state, handle, slug).await?;
        if !matches!(
            authorize(state, cookies, &plan, &access).await?,
            Authorization::Allowed
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
        let mut headers = secure_headers();
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
    Path((handle, slug)): Path<(String, String)>,
) -> Response {
    record_read(&state, &cookies, &handle, &slug, Access::Canonical).await
}

async fn read_shared(
    State(state): State<AppState>,
    cookies: Cookies,
    Path((handle, slug, access)): Path<(String, String, String)>,
) -> Response {
    record_read(&state, &cookies, &handle, &slug, Access::Shared(&access)).await
}

async fn record_read(
    state: &AppState,
    cookies: &Cookies,
    handle: &str,
    slug: &str,
    access: Access<'_>,
) -> Response {
    let result = async {
        let plan = load_plan(state, handle, slug).await?;
        if !matches!(
            authorize(state, cookies, &plan, &access).await?,
            Authorization::Allowed
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

enum Authorization {
    Allowed,
    LoginRequired,
    Hidden,
}

async fn authorize(
    state: &AppState,
    cookies: &Cookies,
    plan: &WebPlan,
    access: &Access<'_>,
) -> Result<Authorization, ApiError> {
    let user = web_user(state, cookies).await?;
    if user.as_ref().is_some_and(|user| user.id == plan.user_id) {
        return Ok(Authorization::Allowed);
    }
    match plan.visibility {
        Visibility::Public => Ok(Authorization::Allowed),
        Visibility::Private => Ok(if user.is_some() {
            Authorization::Hidden
        } else {
            Authorization::LoginRequired
        }),
        Visibility::Unlisted => match (access, &plan.unlisted_token_hash) {
            (Access::Shared(token), Some(expected)) if hash_secret(token) == *expected => {
                Ok(Authorization::Allowed)
            }
            _ => Ok(Authorization::Hidden),
        },
    }
}

async fn load_plan(state: &AppState, handle: &str, slug: &str) -> Result<WebPlan, ApiError> {
    let row = sqlx::query(
        "SELECT plans.id, plans.user_id, plans.title, plans.slug, plans.visibility,
                plans.unlisted_token_hash, users.handle, plan_bundles.object_prefix,
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
    Ok(WebPlan {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        title: row.try_get("title")?,
        handle: row.try_get("handle")?,
        slug: row.try_get("slug")?,
        visibility: row
            .try_get::<String, _>("visibility")?
            .parse()
            .map_err(ApiError::internal)?,
        unlisted_token_hash: row.try_get("unlisted_token_hash")?,
        object_prefix: row.try_get("object_prefix")?,
        manifest: serde_json::from_value(row.try_get("manifest")?).map_err(ApiError::internal)?,
    })
}

fn request_base(plan: &WebPlan, access: &Access<'_>) -> String {
    match access {
        Access::Canonical => format!("/@{}/{}", plan.handle, plan.slug),
        Access::Shared(token) => format!("/@{}/{}/~{}", plan.handle, plan.slug, token),
    }
}

async fn runtime() -> impl IntoResponse {
    let mut headers = secure_headers();
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
    let mut headers = secure_headers();
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

fn secure_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers
}

fn shell(title: &str, read_url: &str, fragment: &str) -> String {
    let title = escape_html(title);
    let read_url = escape_html(read_url);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex,nofollow\"><meta name=\"brume-read-url\" content=\"{read_url}\"><title>{title}</title><link rel=\"stylesheet\" href=\"/_brume/theme.css\"><script type=\"module\" src=\"/_brume/runtime.js\"></script></head><body><div class=\"brume-shell\"><header class=\"brume-topbar\"><a href=\"/\">Brume</a><button class=\"brume-theme-toggle\" data-brume-theme-toggle type=\"button\">Theme</button></header>{fragment}</div></body></html>"
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
