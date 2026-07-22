use axum::{extract::Request, response::Response};

use crate::{deployments, state::AppState, tunnels};

pub async fn serve(state: AppState, public_label: String, request: Request) -> Response {
    if state.tunnels.contains_label(&public_label) {
        return tunnels::relay_request(state, public_label, request).await;
    }
    let method = request.method().clone();
    let request_uri = request.uri().clone();
    deployments::serve_public(state, public_label, method, request_uri).await
}
