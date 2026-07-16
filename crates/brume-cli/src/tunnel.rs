use std::{collections::HashMap, io, time::Duration};

use anyhow::{Context, Result, anyhow, bail};
use brume_core::{
    TUNNEL_BODY_CHUNK_BYTES, TUNNEL_MAX_FRAME_BYTES, TUNNEL_MAX_REQUESTS, TunnelFrame,
    TunnelHeader, TunnelMessage, TunnelWebSocketMessage,
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt, stream};
use http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{
        Error as WebSocketError, Message as WebSocketMessage,
        client::IntoClientRequest,
        protocol::{CloseFrame, WebSocketConfig, frame::coding::CloseCode},
    },
};
use url::Url;
use uuid::Uuid;

const CONTROL_QUEUE_SIZE: usize = 256;
const LOCAL_EVENT_QUEUE_SIZE: usize = 16;

pub async fn run(base_url: &str, token: String, port: u16, slug: &str) -> Result<()> {
    if port == 0 {
        bail!("tunnel port must be between 1 and 65535");
    }
    let endpoint = tunnel_endpoint(base_url, slug)?;
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("brume-cli/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut reconnect_delay = Duration::from_secs(1);

    loop {
        match run_session(&endpoint, &token, port, http.clone()).await {
            Ok(SessionEnd::Stopped) => return Ok(()),
            Ok(SessionEnd::Replaced) => {
                bail!("this tunnel was replaced by a newer connection using the same URL")
            }
            Ok(SessionEnd::Fatal(message)) => bail!(message),
            Ok(SessionEnd::Disconnected) => {
                eprintln!(
                    "Tunnel disconnected; reconnecting in {}s",
                    reconnect_delay.as_secs()
                );
                reconnect_delay = Duration::from_secs(1);
            }
            Err(error) => {
                eprintln!(
                    "Could not connect to Brume Tunnel: {error:#}; retrying in {}s",
                    reconnect_delay.as_secs()
                );
            }
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(reconnect_delay) => {}
        }
        reconnect_delay = (reconnect_delay * 2).min(Duration::from_secs(30));
    }
}

enum SessionEnd {
    Stopped,
    Replaced,
    Disconnected,
    Fatal(String),
}

async fn run_session(
    endpoint: &Url,
    token: &str,
    port: u16,
    http: reqwest::Client,
) -> Result<SessionEnd> {
    let mut request = endpoint.as_str().into_client_request()?;
    request.headers_mut().insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))
            .context("building tunnel authorization header")?,
    );
    request.headers_mut().insert(
        header::USER_AGENT,
        HeaderValue::from_static(concat!("brume-cli/", env!("CARGO_PKG_VERSION"))),
    );
    let config = WebSocketConfig::default()
        .write_buffer_size(0)
        .max_write_buffer_size(TUNNEL_MAX_FRAME_BYTES * 4)
        .max_message_size(Some(TUNNEL_MAX_FRAME_BYTES))
        .max_frame_size(Some(TUNNEL_MAX_FRAME_BYTES));
    let (control, _) = match connect_async_with_config(request, Some(config), true).await {
        Ok(connected) => connected,
        Err(WebSocketError::Http(response)) if response.status().is_client_error() => {
            return Ok(SessionEnd::Fatal(format!(
                "Brume Tunnel rejected the connection with {}",
                response.status()
            )));
        }
        Err(error) => return Err(error.into()),
    };

    let (mut control_tx, mut control_rx) = control.split();
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<TunnelFrame>(CONTROL_QUEUE_SIZE);
    let (completed_tx, mut completed_rx) = mpsc::unbounded_channel::<Uuid>();
    let mut active = HashMap::<Uuid, mpsc::Sender<LocalEvent>>::new();
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            frame = outbound_rx.recv() => {
                let Some(frame) = frame else { return Ok(SessionEnd::Disconnected) };
                let encoded = frame.encode()?;
                if control_tx.send(WebSocketMessage::Binary(encoded.into())).await.is_err() {
                    return Ok(SessionEnd::Disconnected);
                }
            }
            incoming = control_rx.next() => {
                let Some(incoming) = incoming else { return Ok(SessionEnd::Disconnected) };
                match incoming {
                    Ok(WebSocketMessage::Binary(encoded)) => {
                        let frame = TunnelFrame::decode(&encoded)?;
                        match frame.message {
                            TunnelMessage::Welcome { public_url } => {
                                println!("Forwarding {public_url} -> http://127.0.0.1:{port}");
                                println!("This URL is public. Press Ctrl-C to stop.");
                            }
                            TunnelMessage::Replaced => return Ok(SessionEnd::Replaced),
                            TunnelMessage::RequestStart {
                                request_id,
                                method,
                                path_and_query,
                                headers,
                                websocket,
                            } => {
                                if active.len() >= TUNNEL_MAX_REQUESTS || active.contains_key(&request_id) {
                                    send_frame(&outbound_tx, TunnelMessage::ResponseError {
                                        request_id,
                                        message: "local tunnel concurrency limit reached".to_owned(),
                                    }).await?;
                                    continue;
                                }
                                let (events_tx, events_rx) = mpsc::channel(LOCAL_EVENT_QUEUE_SIZE);
                                active.insert(request_id, events_tx);
                                let outbound = outbound_tx.clone();
                                let completed = completed_tx.clone();
                                let http = http.clone();
                                tokio::spawn(async move {
                                    let request = LocalRequest {
                                        request_id,
                                        port,
                                        method,
                                        path_and_query,
                                        headers,
                                    };
                                    let started = std::time::Instant::now();
                                    let result = if websocket {
                                        proxy_websocket(&request, events_rx, &outbound).await
                                    } else {
                                        proxy_http(&request, events_rx, &outbound, http).await
                                    };
                                    match result {
                                        Ok(status) => eprintln!(
                                            "{} {} -> {status} ({}ms)",
                                            request.method,
                                            request.path_and_query,
                                            started.elapsed().as_millis()
                                        ),
                                        Err(LocalRequestError::Cancelled) => {}
                                        Err(LocalRequestError::Failed(error)) => {
                                            eprintln!(
                                                "{} {} -> local error: {error:#}",
                                                request.method, request.path_and_query
                                            );
                                            let _ = send_frame(&outbound, TunnelMessage::ResponseError {
                                                request_id: request.request_id,
                                                message: error.to_string(),
                                            }).await;
                                        }
                                    }
                                    let _ = completed.send(request.request_id);
                                });
                            }
                            TunnelMessage::RequestBody { request_id, bytes } => {
                                dispatch_event(&mut active, request_id, LocalEvent::Body(bytes)).await;
                            }
                            TunnelMessage::RequestEnd { request_id } => {
                                dispatch_event(&mut active, request_id, LocalEvent::End).await;
                            }
                            TunnelMessage::RequestCancel { request_id } => {
                                dispatch_event(&mut active, request_id, LocalEvent::Cancel).await;
                            }
                            TunnelMessage::WebSocketMessage { request_id, message } => {
                                dispatch_event(
                                    &mut active,
                                    request_id,
                                    LocalEvent::WebSocket(message),
                                )
                                .await;
                            }
                            TunnelMessage::ResponseStart { .. }
                            | TunnelMessage::ResponseBody { .. }
                            | TunnelMessage::ResponseEnd { .. }
                            | TunnelMessage::ResponseError { .. } => {
                                return Ok(SessionEnd::Fatal(
                                    "server sent an invalid tunnel protocol message".to_owned(),
                                ));
                            }
                        }
                    }
                    Ok(WebSocketMessage::Ping(bytes)) => {
                        if control_tx.send(WebSocketMessage::Pong(bytes)).await.is_err() {
                            return Ok(SessionEnd::Disconnected);
                        }
                    }
                    Ok(WebSocketMessage::Pong(_)) => {}
                    Ok(WebSocketMessage::Close(_)) | Err(_) => return Ok(SessionEnd::Disconnected),
                    Ok(WebSocketMessage::Text(_)) | Ok(WebSocketMessage::Frame(_)) => {
                        return Ok(SessionEnd::Fatal(
                            "server sent an invalid WebSocket message".to_owned(),
                        ));
                    }
                }
            }
            Some(request_id) = completed_rx.recv() => {
                active.remove(&request_id);
            }
            _ = &mut shutdown => {
                let _ = control_tx.send(WebSocketMessage::Close(None)).await;
                return Ok(SessionEnd::Stopped);
            }
        }
    }
}

enum LocalEvent {
    Body(Vec<u8>),
    End,
    Cancel,
    WebSocket(TunnelWebSocketMessage),
}

struct LocalRequest {
    request_id: Uuid,
    port: u16,
    method: String,
    path_and_query: String,
    headers: Vec<TunnelHeader>,
}

enum LocalRequestError {
    Cancelled,
    Failed(anyhow::Error),
}

impl From<anyhow::Error> for LocalRequestError {
    fn from(error: anyhow::Error) -> Self {
        Self::Failed(error)
    }
}

async fn dispatch_event(
    active: &mut HashMap<Uuid, mpsc::Sender<LocalEvent>>,
    request_id: Uuid,
    event: LocalEvent,
) {
    let sender = active.get(&request_id).cloned();
    if let Some(sender) = sender
        && sender.send(event).await.is_err()
    {
        active.remove(&request_id);
    }
}

async fn proxy_http(
    request: &LocalRequest,
    events: mpsc::Receiver<LocalEvent>,
    outbound: &mpsc::Sender<TunnelFrame>,
    http: reqwest::Client,
) -> Result<u16, LocalRequestError> {
    let method = Method::from_bytes(request.method.as_bytes())
        .context("parsing forwarded HTTP method")
        .map_err(LocalRequestError::Failed)?;
    let target = format!(
        "http://127.0.0.1:{}{}",
        request.port, request.path_and_query
    );
    let headers =
        local_headers(request.headers.clone(), request.port).map_err(LocalRequestError::Failed)?;
    let (body_tx, body_rx) = mpsc::channel::<Bytes>(LOCAL_EVENT_QUEUE_SIZE);
    let (cancel_tx, mut cancel_rx) = watch::channel(false);
    let cancel_guard = cancel_tx.clone();
    tokio::spawn(pump_http_events(events, body_tx, cancel_tx));
    let body_stream = stream::unfold(body_rx, |mut receiver| async move {
        receiver
            .recv()
            .await
            .map(|bytes| (Ok::<Bytes, io::Error>(bytes), receiver))
    });
    let local_call = http
        .request(method, target)
        .headers(headers)
        .body(reqwest::Body::wrap_stream(body_stream));
    let response = tokio::select! {
        response = local_call.send() => response
            .context("connecting to the local HTTP service")
            .map_err(LocalRequestError::Failed)?,
        _ = wait_for_cancel(&mut cancel_rx) => return Err(LocalRequestError::Cancelled),
    };
    let status = response.status().as_u16();
    send_frame(
        outbound,
        TunnelMessage::ResponseStart {
            request_id: request.request_id,
            status,
            headers: headers_to_tunnel(response.headers()),
        },
    )
    .await
    .map_err(LocalRequestError::Failed)?;

    let mut response_body = response.bytes_stream();
    loop {
        tokio::select! {
            chunk = response_body.next() => {
                let Some(chunk) = chunk else { break };
                let mut chunk = chunk
                    .context("reading the local HTTP response")
                    .map_err(LocalRequestError::Failed)?;
                while !chunk.is_empty() {
                    let length = chunk.len().min(TUNNEL_BODY_CHUNK_BYTES);
                    send_frame(outbound, TunnelMessage::ResponseBody {
                        request_id: request.request_id,
                        bytes: chunk.split_to(length).to_vec(),
                    })
                    .await
                    .map_err(LocalRequestError::Failed)?;
                }
            }
            _ = wait_for_cancel(&mut cancel_rx) => return Err(LocalRequestError::Cancelled),
        }
    }
    drop(cancel_guard);
    send_frame(
        outbound,
        TunnelMessage::ResponseEnd {
            request_id: request.request_id,
        },
    )
    .await
    .map_err(LocalRequestError::Failed)?;
    Ok(status)
}

async fn pump_http_events(
    mut events: mpsc::Receiver<LocalEvent>,
    body: mpsc::Sender<Bytes>,
    cancelled: watch::Sender<bool>,
) {
    while let Some(event) = events.recv().await {
        match event {
            LocalEvent::Body(bytes) => {
                if body.send(Bytes::from(bytes)).await.is_err() {
                    return;
                }
            }
            LocalEvent::End => return,
            LocalEvent::Cancel => {
                let _ = cancelled.send(true);
                return;
            }
            LocalEvent::WebSocket(_) => {}
        }
    }
    let _ = cancelled.send(true);
}

async fn wait_for_cancel(cancelled: &mut watch::Receiver<bool>) {
    loop {
        if *cancelled.borrow() {
            return;
        }
        if cancelled.changed().await.is_err() {
            std::future::pending::<()>().await;
        }
    }
}

async fn proxy_websocket(
    request: &LocalRequest,
    mut events: mpsc::Receiver<LocalEvent>,
    outbound: &mpsc::Sender<TunnelFrame>,
) -> Result<u16, LocalRequestError> {
    let endpoint = format!("ws://127.0.0.1:{}{}", request.port, request.path_and_query);
    let mut websocket_request = endpoint
        .into_client_request()
        .context("building the local WebSocket request")
        .map_err(LocalRequestError::Failed)?;
    append_local_websocket_headers(websocket_request.headers_mut(), request.headers.clone())
        .map_err(LocalRequestError::Failed)?;
    let config = WebSocketConfig::default()
        .write_buffer_size(0)
        .max_write_buffer_size(TUNNEL_MAX_FRAME_BYTES * 4)
        .max_message_size(None)
        .max_frame_size(None);
    let (socket, response) = connect_async_with_config(websocket_request, Some(config), true)
        .await
        .context("connecting to the local WebSocket service")
        .map_err(LocalRequestError::Failed)?;
    let status = response.status().as_u16();
    send_frame(
        outbound,
        TunnelMessage::ResponseStart {
            request_id: request.request_id,
            status,
            headers: headers_to_tunnel(response.headers()),
        },
    )
    .await
    .map_err(LocalRequestError::Failed)?;

    let (mut socket_tx, mut socket_rx) = socket.split();
    loop {
        tokio::select! {
            event = events.recv() => {
                match event {
                    Some(LocalEvent::WebSocket(message)) => {
                        let close = matches!(message, TunnelWebSocketMessage::Close { .. });
                        socket_tx
                            .send(tunnel_to_websocket(message))
                            .await
                            .context("writing to the local WebSocket")
                            .map_err(LocalRequestError::Failed)?;
                        if close { break; }
                    }
                    Some(LocalEvent::Cancel) | None => return Err(LocalRequestError::Cancelled),
                    Some(LocalEvent::Body(_)) | Some(LocalEvent::End) => {}
                }
            }
            message = socket_rx.next() => {
                let Some(message) = message else { break };
                let message = message
                    .context("reading from the local WebSocket")
                    .map_err(LocalRequestError::Failed)?;
                if matches!(message, WebSocketMessage::Frame(_)) {
                    continue;
                }
                let close = matches!(message, WebSocketMessage::Close(_));
                send_frame(outbound, TunnelMessage::WebSocketMessage {
                    request_id: request.request_id,
                    message: websocket_to_tunnel(message),
                })
                .await
                .map_err(LocalRequestError::Failed)?;
                if close { break; }
            }
        }
    }
    send_frame(
        outbound,
        TunnelMessage::ResponseEnd {
            request_id: request.request_id,
        },
    )
    .await
    .map_err(LocalRequestError::Failed)?;
    Ok(status)
}

fn tunnel_endpoint(base_url: &str, slug: &str) -> Result<Url> {
    let mut endpoint = Url::parse(base_url)?.join("/")?;
    let scheme = match endpoint.scheme() {
        "http" => "ws",
        "https" => "wss",
        scheme => bail!("unsupported Brume URL scheme `{scheme}`"),
    };
    endpoint
        .set_scheme(scheme)
        .map_err(|_| anyhow!("could not build the Brume Tunnel WebSocket URL"))?;
    endpoint.set_path(&format!("/api/v1/tunnels/{slug}/connect"));
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    Ok(endpoint)
}

fn local_headers(headers: Vec<TunnelHeader>, port: u16) -> Result<HeaderMap> {
    let mut headers = tunnel_to_headers(headers)?;
    headers.insert(
        header::HOST,
        HeaderValue::from_str(&format!("127.0.0.1:{port}"))?,
    );
    Ok(headers)
}

fn append_local_websocket_headers(
    target: &mut HeaderMap,
    headers: Vec<TunnelHeader>,
) -> Result<()> {
    for header in headers {
        let name = HeaderName::from_bytes(header.name.as_bytes())?;
        if matches!(
            name.as_str(),
            "host"
                | "connection"
                | "upgrade"
                | "sec-websocket-key"
                | "sec-websocket-version"
                | "sec-websocket-extensions"
                | "sec-websocket-accept"
        ) {
            continue;
        }
        target.append(name, HeaderValue::from_bytes(&header.value)?);
    }
    Ok(())
}

fn tunnel_to_headers(headers: Vec<TunnelHeader>) -> Result<HeaderMap> {
    let mut decoded = HeaderMap::new();
    for header in headers {
        decoded.append(
            HeaderName::from_bytes(header.name.as_bytes())?,
            HeaderValue::from_bytes(&header.value)?,
        );
    }
    Ok(decoded)
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

async fn send_frame(outbound: &mpsc::Sender<TunnelFrame>, message: TunnelMessage) -> Result<()> {
    let frame = TunnelFrame::new(message);
    frame.encode()?;
    outbound
        .send(frame)
        .await
        .map_err(|_| anyhow!("the Brume Tunnel control connection closed"))
}

fn tunnel_to_websocket(message: TunnelWebSocketMessage) -> WebSocketMessage {
    match message {
        TunnelWebSocketMessage::Text(text) => WebSocketMessage::Text(text.into()),
        TunnelWebSocketMessage::Binary(bytes) => WebSocketMessage::Binary(bytes.into()),
        TunnelWebSocketMessage::Ping(bytes) => WebSocketMessage::Ping(bytes.into()),
        TunnelWebSocketMessage::Pong(bytes) => WebSocketMessage::Pong(bytes.into()),
        TunnelWebSocketMessage::Close { code, reason } => {
            WebSocketMessage::Close(code.map(|code| CloseFrame {
                code: CloseCode::from(code),
                reason: reason.into(),
            }))
        }
    }
}

fn websocket_to_tunnel(message: WebSocketMessage) -> TunnelWebSocketMessage {
    match message {
        WebSocketMessage::Text(text) => TunnelWebSocketMessage::Text(text.to_string()),
        WebSocketMessage::Binary(bytes) => TunnelWebSocketMessage::Binary(bytes.to_vec()),
        WebSocketMessage::Ping(bytes) => TunnelWebSocketMessage::Ping(bytes.to_vec()),
        WebSocketMessage::Pong(bytes) => TunnelWebSocketMessage::Pong(bytes.to_vec()),
        WebSocketMessage::Close(frame) => TunnelWebSocketMessage::Close {
            code: frame.as_ref().map(|frame| u16::from(frame.code)),
            reason: frame
                .map(|frame| frame.reason.to_string())
                .unwrap_or_default(),
        },
        WebSocketMessage::Frame(_) => unreachable!("raw frames are filtered by the caller"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_websocket_endpoint_from_api_url() {
        assert_eq!(
            tunnel_endpoint("https://api.brume.dev", "my-app")
                .unwrap()
                .as_str(),
            "wss://api.brume.dev/api/v1/tunnels/my-app/connect"
        );
        assert_eq!(
            tunnel_endpoint("http://127.0.0.1:8080", "my-app")
                .unwrap()
                .as_str(),
            "ws://127.0.0.1:8080/api/v1/tunnels/my-app/connect"
        );
    }

    #[test]
    fn rewrites_local_host_without_losing_forwarding_headers() {
        let headers = local_headers(
            vec![TunnelHeader {
                name: "x-forwarded-prefix".to_owned(),
                value: b"/paul/app".to_vec(),
            }],
            3000,
        )
        .unwrap();

        assert_eq!(headers[header::HOST], "127.0.0.1:3000");
        assert_eq!(headers["x-forwarded-prefix"], "/paul/app");
    }
}
