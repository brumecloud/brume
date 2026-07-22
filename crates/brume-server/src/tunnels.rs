use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{FromRequestParts, Query, Request, State, WebSocketUpgrade, ws},
    http::{
        HeaderMap, HeaderName, HeaderValue, Method, StatusCode,
        header::{self, HOST},
    },
    response::{IntoResponse, Response},
    routing::get,
};
use brume_core::{
    TUNNEL_BODY_CHUNK_BYTES, TUNNEL_MAX_FRAME_BYTES, TUNNEL_MAX_REQUESTS, TunnelFrame,
    TunnelHeader, TunnelMessage, TunnelWebSocketMessage,
};
use futures::{SinkExt, StreamExt, stream};
use serde::Deserialize;
use tokio::sync::{Semaphore, mpsc, watch};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::ApiError,
    state::AppState,
    util::{public_label, random_public_id},
};

const CONTROL_QUEUE_SIZE: usize = 256;
const RESPONSE_QUEUE_SIZE: usize = 16;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(60);

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/tunnels/connect", get(connect_tunnel))
}

#[derive(Clone, Default)]
pub struct TunnelRegistry {
    inner: Arc<Mutex<RegistryState>>,
}

#[derive(Default)]
struct RegistryState {
    by_owner: HashMap<(Uuid, String), Arc<TunnelConnection>>,
    by_public_label: HashMap<String, Arc<TunnelConnection>>,
    claims: HashMap<String, (Uuid, String)>,
}

impl TunnelRegistry {
    pub(crate) fn contains_label(&self, label: &str) -> bool {
        let state = self.inner.lock().expect("tunnel registry lock poisoned");
        state.by_public_label.contains_key(label) || state.claims.contains_key(label)
    }

    fn claim(&self, label: &str, user_id: Uuid, slug: &str) -> bool {
        let mut state = self.inner.lock().expect("tunnel registry lock poisoned");
        let owner = (user_id, slug.to_owned());
        if state
            .by_public_label
            .get(label)
            .is_some_and(|connection| (connection.user_id, connection.slug.clone()) != owner)
            || state
                .claims
                .get(label)
                .is_some_and(|claimed| claimed != &owner)
        {
            return false;
        }
        state.claims.insert(label.to_owned(), owner);
        true
    }

    fn register(&self, connection: Arc<TunnelConnection>) {
        let owner_key = (connection.user_id, connection.slug.clone());
        let public_key = connection.public_label.clone();
        let previous = {
            let mut state = self.inner.lock().expect("tunnel registry lock poisoned");
            let previous = state.by_owner.insert(owner_key, connection.clone());
            if let Some(previous) = &previous {
                state.by_public_label.remove(&previous.public_label);
            }
            state.claims.remove(&public_key);
            state.by_public_label.insert(public_key, connection);
            previous
        };
        if let Some(previous) = previous {
            let _ = previous.replaced.send(true);
        }
    }

    fn get(&self, label: &str) -> Option<Arc<TunnelConnection>> {
        self.inner
            .lock()
            .expect("tunnel registry lock poisoned")
            .by_public_label
            .get(label)
            .cloned()
    }

    fn remove(&self, connection: &TunnelConnection) {
        let owner_key = (connection.user_id, connection.slug.clone());
        let public_key = connection.public_label.clone();
        let mut state = self.inner.lock().expect("tunnel registry lock poisoned");
        if state
            .by_owner
            .get(&owner_key)
            .is_some_and(|current| current.id == connection.id)
        {
            state.by_owner.remove(&owner_key);
        }
        if state
            .by_public_label
            .get(&public_key)
            .is_some_and(|current| current.id == connection.id)
        {
            state.by_public_label.remove(&public_key);
        }
    }
}

struct TunnelConnection {
    id: Uuid,
    user_id: Uuid,
    handle: String,
    slug: String,
    public_label: String,
    outbound: mpsc::Sender<TunnelFrame>,
    replaced: watch::Sender<bool>,
    pending: Mutex<HashMap<Uuid, mpsc::Sender<TunnelMessage>>>,
    requests: Arc<Semaphore>,
}

impl TunnelConnection {
    fn new(
        user: AuthUser,
        slug: String,
        public_label: String,
        outbound: mpsc::Sender<TunnelFrame>,
        replaced: watch::Sender<bool>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user.id,
            handle: user.handle,
            slug,
            public_label,
            outbound,
            replaced,
            pending: Mutex::new(HashMap::new()),
            requests: Arc::new(Semaphore::new(TUNNEL_MAX_REQUESTS)),
        }
    }

    async fn begin(
        self: &Arc<Self>,
        message: TunnelMessage,
    ) -> Result<PendingRequest, RelayFailure> {
        let request_id = message
            .request_id()
            .expect("request start has a request ID");
        let permit = self
            .requests
            .clone()
            .try_acquire_owned()
            .map_err(|_| RelayFailure::Saturated)?;
        let (response_tx, response_rx) = mpsc::channel(RESPONSE_QUEUE_SIZE);
        self.pending
            .lock()
            .expect("pending request lock poisoned")
            .insert(request_id, response_tx);
        let pending = PendingRequest {
            request_id,
            response_rx,
            connection: self.clone(),
            completed: false,
            _permit: permit,
        };
        self.send(message)
            .await
            .map_err(|_| RelayFailure::Unavailable)?;
        Ok(pending)
    }

    async fn send(&self, message: TunnelMessage) -> Result<(), ()> {
        let frame = TunnelFrame::new(message);
        frame.encode().map_err(|_| ())?;
        self.outbound.send(frame).await.map_err(|_| ())
    }

    async fn dispatch(&self, message: TunnelMessage) -> Result<(), ()> {
        if !matches!(
            message,
            TunnelMessage::ResponseStart { .. }
                | TunnelMessage::ResponseBody { .. }
                | TunnelMessage::ResponseEnd { .. }
                | TunnelMessage::ResponseError { .. }
                | TunnelMessage::WebSocketMessage { .. }
        ) {
            return Err(());
        }
        let request_id = message.request_id().ok_or(())?;
        let sender = self
            .pending
            .lock()
            .expect("pending request lock poisoned")
            .get(&request_id)
            .cloned();
        if let Some(sender) = sender
            && sender.send(message).await.is_err()
        {
            self.pending
                .lock()
                .expect("pending request lock poisoned")
                .remove(&request_id);
        }
        Ok(())
    }

    fn close_pending(&self) {
        self.pending
            .lock()
            .expect("pending request lock poisoned")
            .clear();
    }
}

struct PendingRequest {
    request_id: Uuid,
    response_rx: mpsc::Receiver<TunnelMessage>,
    connection: Arc<TunnelConnection>,
    completed: bool,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PendingRequest {
    async fn receive(&mut self) -> Option<TunnelMessage> {
        self.response_rx.recv().await
    }

    async fn send(&self, message: TunnelMessage) -> Result<(), ()> {
        self.connection.send(message).await
    }

    fn complete(&mut self) {
        self.completed = true;
    }
}

impl Drop for PendingRequest {
    fn drop(&mut self) {
        self.connection
            .pending
            .lock()
            .expect("pending request lock poisoned")
            .remove(&self.request_id);
        if !self.completed {
            let _ =
                self.connection
                    .outbound
                    .try_send(TunnelFrame::new(TunnelMessage::RequestCancel {
                        request_id: self.request_id,
                    }));
        }
    }
}

#[derive(Debug)]
enum RelayFailure {
    NotFound,
    Saturated,
    Unavailable,
}

impl RelayFailure {
    fn response(self) -> Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Tunnel not found").into_response(),
            Self::Saturated => {
                (StatusCode::SERVICE_UNAVAILABLE, "Tunnel is saturated").into_response()
            }
            Self::Unavailable => {
                (StatusCode::BAD_GATEWAY, "Tunnel connection failed").into_response()
            }
        }
    }
}

#[derive(Deserialize)]
struct ConnectParameters {
    slug: Option<String>,
}

async fn connect_tunnel(
    State(state): State<AppState>,
    user: AuthUser,
    Query(parameters): Query<ConnectParameters>,
    websocket: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let requested_slug = parameters.slug;
    if let Some(slug) = &requested_slug {
        validate_slug(slug)?;
    }
    let generated = requested_slug.is_none();
    let mut candidate = requested_slug.unwrap_or_else(random_public_id);
    let (slug, public_label) = {
        let _guard = state.public_endpoints.lock().await;
        loop {
            let label = public_label(&candidate, &user.handle).ok_or_else(|| {
                ApiError::bad_request("Tunnel URL slug is too long for this user handle")
            })?;
            let deployment_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(
                SELECT 1 FROM deployments
                WHERE public_label = $1 AND status = 'active'
            )",
            )
            .bind(&label)
            .fetch_one(&state.database)
            .await?;
            if !deployment_exists && state.tunnels.claim(&label, user.id, &candidate) {
                break (candidate, label);
            }
            if !generated {
                return Err(ApiError::public_url_conflict(
                    "This public URL is already used by another tunnel or deployment",
                ));
            }
            candidate = random_public_id();
        }
    };
    let public_url = state.config.public_url(&public_label);
    Ok(websocket
        .max_message_size(TUNNEL_MAX_FRAME_BYTES)
        .max_frame_size(TUNNEL_MAX_FRAME_BYTES)
        .write_buffer_size(0)
        .max_write_buffer_size(TUNNEL_MAX_FRAME_BYTES * 4)
        .on_upgrade(move |socket| {
            run_control_connection(state, user, slug, public_label, public_url, socket)
        }))
}

async fn run_control_connection(
    state: AppState,
    user: AuthUser,
    slug: String,
    public_label: String,
    public_url: String,
    socket: ws::WebSocket,
) {
    let (outbound_tx, mut outbound_rx) = mpsc::channel(CONTROL_QUEUE_SIZE);
    let (replaced_tx, mut replaced_rx) = watch::channel(false);
    let connection = Arc::new(TunnelConnection::new(
        user,
        slug,
        public_label,
        outbound_tx,
        replaced_tx,
    ));
    state.tunnels.register(connection.clone());
    let _ = connection.send(TunnelMessage::Welcome { public_url }).await;

    let (mut websocket_tx, mut websocket_rx) = socket.split();
    let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_seen = Instant::now();

    loop {
        tokio::select! {
            outbound = outbound_rx.recv() => {
                let Some(outbound) = outbound else { break };
                let encoded = match outbound.encode() {
                    Ok(encoded) => encoded,
                    Err(error) => {
                        tracing::warn!(%error, tunnel_id = %connection.id, "could not encode tunnel frame");
                        break;
                    }
                };
                if websocket_tx.send(ws::Message::Binary(encoded.into())).await.is_err() {
                    break;
                }
            }
            incoming = websocket_rx.next() => {
                let Some(incoming) = incoming else { break };
                match incoming {
                    Ok(ws::Message::Binary(encoded)) => {
                        last_seen = Instant::now();
                        let frame = match TunnelFrame::decode(&encoded) {
                            Ok(frame) => frame,
                            Err(error) => {
                                tracing::warn!(%error, tunnel_id = %connection.id, "invalid tunnel frame");
                                break;
                            }
                        };
                        if connection.dispatch(frame.message).await.is_err() {
                            tracing::warn!(tunnel_id = %connection.id, "invalid client tunnel message");
                            break;
                        }
                    }
                    Ok(ws::Message::Ping(bytes)) => {
                        last_seen = Instant::now();
                        if websocket_tx.send(ws::Message::Pong(bytes)).await.is_err() {
                            break;
                        }
                    }
                    Ok(ws::Message::Pong(_)) => last_seen = Instant::now(),
                    Ok(ws::Message::Close(_)) | Err(_) => break,
                    Ok(ws::Message::Text(_)) => break,
                }
            }
            _ = heartbeat.tick() => {
                if last_seen.elapsed() > HEARTBEAT_TIMEOUT {
                    break;
                }
                if websocket_tx.send(ws::Message::Ping(Bytes::from_static(b"brume"))).await.is_err() {
                    break;
                }
            }
            changed = replaced_rx.changed() => {
                if changed.is_err() || *replaced_rx.borrow() {
                    if let Ok(encoded) = TunnelFrame::new(TunnelMessage::Replaced).encode() {
                        let _ = websocket_tx.send(ws::Message::Binary(encoded.into())).await;
                    }
                    let _ = websocket_tx.send(ws::Message::Close(None)).await;
                    break;
                }
            }
        }
    }

    state.tunnels.remove(&connection);
    connection.close_pending();
    tracing::info!(
        tunnel_id = %connection.id,
        handle = %connection.handle,
        slug = %connection.slug,
        "tunnel disconnected"
    );
}

pub(crate) async fn relay_request(
    state: AppState,
    public_label: String,
    request: Request,
) -> Response {
    let (mut parts, body) = request.into_parts();
    let Some(connection) = state.tunnels.get(&public_label) else {
        return RelayFailure::NotFound.response();
    };
    let mut path_and_query = parts.uri.path().to_owned();
    if let Some(query) = parts.uri.query() {
        path_and_query.push('?');
        path_and_query.push_str(query);
    }
    let headers = sanitize_request_headers(&parts.headers, &state.config.public_scheme);
    let websocket_requested = is_websocket_request(&parts.method, &parts.headers);
    let websocket = if websocket_requested {
        match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
            Ok(websocket) => Some(websocket),
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "Invalid WebSocket upgrade").into_response();
            }
        }
    } else {
        None
    };
    let request_id = Uuid::now_v7();
    let pending = match connection
        .begin(TunnelMessage::RequestStart {
            request_id,
            method: parts.method.to_string(),
            path_and_query,
            headers: headers_to_tunnel(&headers),
            websocket: websocket_requested,
        })
        .await
    {
        Ok(pending) => pending,
        Err(failure) => return failure.response(),
    };

    if let Some(websocket) = websocket {
        if pending
            .send(TunnelMessage::RequestEnd { request_id })
            .await
            .is_err()
        {
            return RelayFailure::Unavailable.response();
        }
        return relay_websocket(websocket, pending).await;
    }

    let body_connection = connection.clone();
    tokio::spawn(async move {
        forward_request_body(body_connection, request_id, body).await;
    });
    relay_http(pending).await
}

async fn relay_http(mut pending: PendingRequest) -> Response {
    let (status, headers) = loop {
        match pending.receive().await {
            Some(TunnelMessage::ResponseStart {
                status, headers, ..
            }) => break (status, headers),
            Some(TunnelMessage::ResponseError { .. }) | None => {
                pending.complete();
                return RelayFailure::Unavailable.response();
            }
            Some(_) => continue,
        }
    };
    let status = match StatusCode::from_u16(status) {
        Ok(status) if status != StatusCode::SWITCHING_PROTOCOLS => status,
        _ => return RelayFailure::Unavailable.response(),
    };
    let headers = match sanitize_response_headers(headers) {
        Ok(headers) => headers,
        Err(_) => return RelayFailure::Unavailable.response(),
    };

    let response_stream = stream::unfold(Some(pending), |state| async move {
        let mut pending = state?;
        loop {
            match pending.receive().await {
                Some(TunnelMessage::ResponseBody { bytes, .. }) => {
                    return Some((Ok::<Bytes, io::Error>(Bytes::from(bytes)), Some(pending)));
                }
                Some(TunnelMessage::ResponseEnd { .. }) => {
                    pending.complete();
                    return None;
                }
                Some(TunnelMessage::ResponseError { message, .. }) => {
                    pending.complete();
                    return Some((Err(io::Error::other(message)), None));
                }
                Some(_) => continue,
                None => {
                    pending.complete();
                    return Some((
                        Err(io::Error::new(
                            io::ErrorKind::ConnectionReset,
                            "tunnel disconnected before the response completed",
                        )),
                        None,
                    ));
                }
            }
        }
    });
    let mut response = Response::new(Body::from_stream(response_stream.fuse()));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

async fn relay_websocket(mut websocket: WebSocketUpgrade, mut pending: PendingRequest) -> Response {
    let response_headers = loop {
        match pending.receive().await {
            Some(TunnelMessage::ResponseStart {
                status, headers, ..
            }) if status == StatusCode::SWITCHING_PROTOCOLS.as_u16() => break headers,
            Some(TunnelMessage::ResponseError { .. }) | None => {
                pending.complete();
                return RelayFailure::Unavailable.response();
            }
            Some(_) => continue,
        }
    };
    let mut headers = match sanitize_response_headers(response_headers) {
        Ok(headers) => headers,
        Err(_) => return RelayFailure::Unavailable.response(),
    };
    headers.remove(header::SEC_WEBSOCKET_ACCEPT);
    headers.remove(header::SEC_WEBSOCKET_EXTENSIONS);
    if let Some(protocol) = headers.remove(header::SEC_WEBSOCKET_PROTOCOL) {
        let requested = websocket
            .requested_protocols()
            .any(|value| value == protocol);
        if !requested {
            return RelayFailure::Unavailable.response();
        }
        websocket.set_selected_protocol(protocol);
    }
    let mut response = websocket.on_upgrade(move |socket| bridge_websocket(socket, pending));
    for (name, value) in &headers {
        response.headers_mut().append(name.clone(), value.clone());
    }
    response
}

async fn bridge_websocket(socket: ws::WebSocket, mut pending: PendingRequest) {
    let (mut socket_tx, mut socket_rx) = socket.split();
    loop {
        tokio::select! {
            incoming = socket_rx.next() => {
                let Some(Ok(message)) = incoming else { break };
                let close = matches!(message, ws::Message::Close(_));
                let message = TunnelMessage::WebSocketMessage {
                    request_id: pending.request_id,
                    message: websocket_to_tunnel(message),
                };
                if pending.send(message).await.is_err() || close {
                    break;
                }
            }
            outgoing = pending.receive() => {
                match outgoing {
                    Some(TunnelMessage::WebSocketMessage { message, .. }) => {
                        let close = matches!(message, TunnelWebSocketMessage::Close { .. });
                        if socket_tx.send(tunnel_to_websocket(message)).await.is_err() || close {
                            if close {
                                pending.complete();
                            }
                            break;
                        }
                    }
                    Some(TunnelMessage::ResponseEnd { .. }) => {
                        pending.complete();
                        break;
                    }
                    None => break,
                    Some(TunnelMessage::ResponseError { .. }) => {
                        let _ = socket_tx.send(ws::Message::Close(Some(ws::CloseFrame {
                            code: 1011,
                            reason: "Tunnel connection failed".into(),
                        }))).await;
                        pending.complete();
                        break;
                    }
                    Some(_) => {}
                }
            }
        }
    }
}

async fn forward_request_body(connection: Arc<TunnelConnection>, request_id: Uuid, body: Body) {
    let mut body = body.into_data_stream();
    while let Some(chunk) = body.next().await {
        let mut chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => {
                let _ = connection
                    .send(TunnelMessage::RequestCancel { request_id })
                    .await;
                return;
            }
        };
        while !chunk.is_empty() {
            let length = chunk.len().min(TUNNEL_BODY_CHUNK_BYTES);
            let bytes = chunk.split_to(length).to_vec();
            if connection
                .send(TunnelMessage::RequestBody { request_id, bytes })
                .await
                .is_err()
            {
                return;
            }
        }
    }
    let _ = connection
        .send(TunnelMessage::RequestEnd { request_id })
        .await;
}

fn validate_slug(slug: &str) -> Result<(), ApiError> {
    if slug.is_empty()
        || slug.len() > 80
        || slug.starts_with('-')
        || slug.ends_with('-')
        || !slug
            .chars()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-')
    {
        return Err(ApiError::bad_request("Invalid tunnel URL slug"));
    }
    Ok(())
}

fn is_websocket_request(method: &Method, headers: &HeaderMap) -> bool {
    method == Method::GET
        && headers
            .get(header::UPGRADE)
            .is_some_and(|value| value.as_bytes().eq_ignore_ascii_case(b"websocket"))
        && headers.get(header::CONNECTION).is_some_and(|value| {
            value
                .to_str()
                .is_ok_and(|value| value.to_ascii_lowercase().contains("upgrade"))
        })
}

fn sanitize_request_headers(headers: &HeaderMap, scheme: &str) -> HeaderMap {
    let original_host = headers.get(HOST).cloned();
    let mut headers = remove_hop_by_hop(headers);
    headers.remove(HOST);
    headers.remove(header::SEC_WEBSOCKET_KEY);
    headers.remove(header::SEC_WEBSOCKET_VERSION);
    headers.remove(header::SEC_WEBSOCKET_EXTENSIONS);
    headers.remove(header::SEC_WEBSOCKET_ACCEPT);
    strip_brume_cookies(&mut headers);
    if let Some(host) = original_host {
        headers.insert(HeaderName::from_static("x-forwarded-host"), host);
    }
    headers.insert(
        HeaderName::from_static("x-forwarded-proto"),
        HeaderValue::from_str(scheme).expect("configured URL scheme is a valid header value"),
    );
    headers
}

fn strip_brume_cookies(headers: &mut HeaderMap) {
    let cookies = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .map(str::trim)
        .filter(|cookie| {
            cookie
                .split_once('=')
                .is_some_and(|(name, _)| !name.trim().starts_with("brume_"))
        })
        .collect::<Vec<_>>()
        .join("; ");
    headers.remove(header::COOKIE);
    if !cookies.is_empty()
        && let Ok(value) = HeaderValue::from_str(&cookies)
    {
        headers.insert(header::COOKIE, value);
    }
}

fn sanitize_response_headers(headers: Vec<TunnelHeader>) -> Result<HeaderMap, ()> {
    let headers = tunnel_to_headers(headers)?;
    Ok(remove_hop_by_hop(&headers))
}

fn remove_hop_by_hop(headers: &HeaderMap) -> HeaderMap {
    let mut removed = HashSet::new();
    for value in headers.get_all(header::CONNECTION) {
        if let Ok(value) = value.to_str() {
            removed.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_ascii_lowercase),
            );
        }
    }
    let mut filtered = HeaderMap::new();
    for (name, value) in headers {
        if is_hop_by_hop(name) || removed.contains(name.as_str()) {
            continue;
        }
        filtered.append(name, value.clone());
    }
    filtered
}

fn is_hop_by_hop(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "proxy-connection"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn headers_to_tunnel(headers: &HeaderMap) -> Vec<TunnelHeader> {
    headers
        .iter()
        .map(|(name, value)| TunnelHeader {
            name: name.as_str().to_owned(),
            value: value.as_bytes().to_vec(),
        })
        .collect()
}

fn tunnel_to_headers(headers: Vec<TunnelHeader>) -> Result<HeaderMap, ()> {
    let mut decoded = HeaderMap::new();
    for header in headers {
        let name = HeaderName::from_bytes(header.name.as_bytes()).map_err(|_| ())?;
        let value = HeaderValue::from_bytes(&header.value).map_err(|_| ())?;
        decoded.append(name, value);
    }
    Ok(decoded)
}

fn websocket_to_tunnel(message: ws::Message) -> TunnelWebSocketMessage {
    match message {
        ws::Message::Text(text) => TunnelWebSocketMessage::Text(text.to_string()),
        ws::Message::Binary(bytes) => TunnelWebSocketMessage::Binary(bytes.to_vec()),
        ws::Message::Ping(bytes) => TunnelWebSocketMessage::Ping(bytes.to_vec()),
        ws::Message::Pong(bytes) => TunnelWebSocketMessage::Pong(bytes.to_vec()),
        ws::Message::Close(frame) => TunnelWebSocketMessage::Close {
            code: frame.as_ref().map(|frame| frame.code),
            reason: frame
                .map(|frame| frame.reason.to_string())
                .unwrap_or_default(),
        },
    }
}

fn tunnel_to_websocket(message: TunnelWebSocketMessage) -> ws::Message {
    match message {
        TunnelWebSocketMessage::Text(text) => ws::Message::Text(text.into()),
        TunnelWebSocketMessage::Binary(bytes) => ws::Message::Binary(bytes.into()),
        TunnelWebSocketMessage::Ping(bytes) => ws::Message::Ping(bytes.into()),
        TunnelWebSocketMessage::Pong(bytes) => ws::Message::Pong(bytes.into()),
        TunnelWebSocketMessage::Close { code, reason } => {
            ws::Message::Close(code.map(|code| ws::CloseFrame {
                code,
                reason: reason.into(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection(
        user_id: Uuid,
        handle: &str,
        slug: &str,
    ) -> (Arc<TunnelConnection>, watch::Receiver<bool>) {
        let (outbound, _receiver) = mpsc::channel(1);
        let (replaced, receiver) = watch::channel(false);
        (
            Arc::new(TunnelConnection::new(
                AuthUser {
                    id: user_id,
                    handle: handle.to_owned(),
                },
                slug.to_owned(),
                format!("{slug}-{handle}"),
                outbound,
                replaced,
            )),
            receiver,
        )
    }

    #[test]
    fn newest_connection_replaces_old_without_old_cleanup_removing_it() {
        let registry = TunnelRegistry::default();
        let user_id = Uuid::now_v7();
        let (old, old_replaced) = connection(user_id, "paul", "app");
        let (new, _) = connection(user_id, "paul", "app");

        registry.register(old.clone());
        registry.register(new.clone());
        registry.remove(&old);

        assert!(*old_replaced.borrow());
        assert_eq!(registry.get("app-paul").unwrap().id, new.id);
    }

    #[tokio::test]
    async fn connection_rejects_more_than_the_bounded_request_limit() {
        let (outbound, _receiver) = mpsc::channel(TUNNEL_MAX_REQUESTS * 2);
        let (replaced, _) = watch::channel(false);
        let connection = Arc::new(TunnelConnection::new(
            AuthUser {
                id: Uuid::now_v7(),
                handle: "paul".to_owned(),
            },
            "app".to_owned(),
            "app-paul".to_owned(),
            outbound,
            replaced,
        ));
        let mut pending = Vec::new();
        for _ in 0..TUNNEL_MAX_REQUESTS {
            let request_id = Uuid::now_v7();
            pending.push(
                connection
                    .begin(TunnelMessage::RequestStart {
                        request_id,
                        method: "GET".to_owned(),
                        path_and_query: "/".to_owned(),
                        headers: Vec::new(),
                        websocket: false,
                    })
                    .await
                    .unwrap(),
            );
        }

        let overflow = connection
            .begin(TunnelMessage::RequestStart {
                request_id: Uuid::now_v7(),
                method: "GET".to_owned(),
                path_and_query: "/".to_owned(),
                headers: Vec::new(),
                websocket: false,
            })
            .await;

        assert!(matches!(overflow, Err(RelayFailure::Saturated)));
        drop(pending);
    }

    #[test]
    fn request_headers_drop_hop_by_hop_values_and_add_forwarding_context() {
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_static("tunnel.brume.dev"));
        headers.insert(
            header::CONNECTION,
            HeaderValue::from_static("keep-alive, x-drop"),
        );
        headers.insert("x-drop", HeaderValue::from_static("secret"));
        headers.insert("x-keep", HeaderValue::from_static("visible"));
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("session=1; brume_plan_access=secret"),
        );

        let headers = sanitize_request_headers(&headers, "https");

        assert!(!headers.contains_key(header::CONNECTION));
        assert!(!headers.contains_key("x-drop"));
        assert_eq!(headers["x-keep"], "visible");
        assert_eq!(headers["x-forwarded-host"], "tunnel.brume.dev");
        assert!(!headers.contains_key("x-forwarded-prefix"));
        assert_eq!(headers[header::COOKIE], "session=1");
    }

    #[test]
    fn response_preserves_root_location_and_cookie_path() {
        let headers = sanitize_response_headers(vec![
            TunnelHeader {
                name: "location".to_owned(),
                value: b"/login".to_vec(),
            },
            TunnelHeader {
                name: "set-cookie".to_owned(),
                value: b"session=1; Path=/; HttpOnly".to_vec(),
            },
            TunnelHeader {
                name: "set-cookie".to_owned(),
                value: b"theme=dark; Path=/; SameSite=Lax".to_vec(),
            },
        ])
        .unwrap();

        assert_eq!(headers[header::LOCATION], "/login");
        let cookies = headers
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            cookies,
            [
                "session=1; Path=/; HttpOnly",
                "theme=dark; Path=/; SameSite=Lax"
            ]
        );
    }
}
