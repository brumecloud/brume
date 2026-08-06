use axum::{
    Router,
    extract::{FromRequestParts, Request},
    http::header,
    response::{IntoResponse, Response},
};
use tower::ServiceExt;
use tower_cookies::Cookies;

use crate::{access, deployments, review, state::AppState, tunnels};

pub async fn serve(state: AppState, public_label: String, request: Request) -> Response {
    if state.tunnels.contains_label(&public_label) {
        return tunnels::relay_request(state, public_label, request).await;
    }
    if matches!(
        request.uri().path(),
        "/_brume/access/complete" | "/_brume/overlay-state" | "/_brume/overlay.js"
    ) || request.uri().path().starts_with("/_brume/review/")
    {
        return Router::new()
            .merge(access::site_router())
            .merge(review::site_router())
            .with_state(state)
            .oneshot(request)
            .await
            .expect("website access router is infallible");
    }
    let bearer_token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned);
    let (mut parts, _) = request.into_parts();
    let cookies = match Cookies::from_request_parts(&mut parts, &state).await {
        Ok(cookies) => cookies,
        Err(error) => return error.into_response(),
    };
    let method = parts.method;
    let request_uri = parts.uri;
    deployments::serve_public(
        state,
        cookies,
        bearer_token,
        public_label,
        method,
        request_uri,
    )
    .await
}
