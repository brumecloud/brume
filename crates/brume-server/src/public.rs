use axum::{
    Router,
    extract::{Path, Request, State},
    http::{HeaderMap, header::HOST},
    response::{IntoResponse, Response},
    routing::any,
};

use crate::{deployments, state::AppState, tunnels};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{handle}/{slug}", any(root))
        .route("/{handle}/{slug}/", any(root_with_slash))
        .route("/{handle}/{slug}/{*path}", any(path))
}

async fn root(
    State(state): State<AppState>,
    Path((handle, slug)): Path<(String, String)>,
    request: Request,
) -> Response {
    dispatch(state, handle, slug, None, false, request).await
}

async fn root_with_slash(
    State(state): State<AppState>,
    Path((handle, slug)): Path<(String, String)>,
    request: Request,
) -> Response {
    dispatch(state, handle, slug, None, true, request).await
}

async fn path(
    State(state): State<AppState>,
    Path((handle, slug, path)): Path<(String, String, String)>,
    request: Request,
) -> Response {
    dispatch(state, handle, slug, Some(path), true, request).await
}

async fn dispatch(
    state: AppState,
    handle: String,
    slug: String,
    path: Option<String>,
    trailing_slash: bool,
    request: Request,
) -> Response {
    let tunnel_host = host_matches(request.headers(), &state.config.tunnel_public_host);
    let deploy_host = host_matches(request.headers(), &state.config.deploy_public_host);
    if tunnel_host && (!deploy_host || state.tunnels.contains(&handle, &slug)) {
        return tunnels::relay_request(state, handle, slug, request).await;
    }
    if deploy_host {
        let method = request.method().clone();
        let request_uri = request.uri().clone();
        return deployments::serve_public(
            state,
            handle,
            slug,
            path,
            method,
            trailing_slash,
            request_uri,
        )
        .await;
    }
    axum::http::StatusCode::NOT_FOUND.into_response()
}

fn host_matches(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}
