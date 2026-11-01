//! `GET /api/environments/:id/logs`: the live logs of an environment, in Server-Sent Events.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};

use crate::routes::{engine_error_response, failure_response};
use crate::{AppState, session};

/// A comment goes out this often, so that proxies and the browser keep the stream open.
const PING_PERIOD: Duration = Duration::from_secs(15);

/// Opens the engine's log stream on a connection of its own. A refusal answers its status with
/// a JSON body; once open, each batch of lines becomes one `data:` event holding the JSON
/// array, until the engine ends the stream or the browser leaves.
pub(crate) async fn environment_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(environment_id): Path<String>,
) -> Response {
    let stream = match state
        .engine
        .open_log_stream(session::token(&headers), environment_id)
        .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(failure)) => return failure_response(failure),
        Err(error) => return engine_error_response(error),
    };
    let events = futures_util::stream::unfold(stream, |mut stream| async move {
        let lines = stream.next().await?;
        // A JSON string escapes its newlines, so the event holds one `data:` line.
        let data = match serde_json::to_string(&lines) {
            Ok(data) => data,
            Err(error) => {
                tracing::warn!(%error, "cannot encode log lines");
                return None;
            }
        };
        Some((Ok::<_, Infallible>(Event::default().data(data)), stream))
    });
    Sse::new(events)
        .keep_alive(KeepAlive::new().interval(PING_PERIOD).text("ping"))
        .into_response()
}
