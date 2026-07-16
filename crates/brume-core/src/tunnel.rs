use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const TUNNEL_PROTOCOL_VERSION: u16 = 1;
pub const TUNNEL_BODY_CHUNK_BYTES: usize = 64 * 1024;
pub const TUNNEL_MAX_FRAME_BYTES: usize = 128 * 1024;
pub const TUNNEL_MAX_REQUESTS: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelFrame {
    pub version: u16,
    pub message: TunnelMessage,
}

impl TunnelFrame {
    pub fn new(message: TunnelMessage) -> Self {
        Self {
            version: TUNNEL_PROTOCOL_VERSION,
            message,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, TunnelProtocolError> {
        let encoded = rmp_serde::to_vec_named(self)?;
        if encoded.len() > TUNNEL_MAX_FRAME_BYTES {
            return Err(TunnelProtocolError::FrameTooLarge(encoded.len()));
        }
        Ok(encoded)
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, TunnelProtocolError> {
        if encoded.len() > TUNNEL_MAX_FRAME_BYTES {
            return Err(TunnelProtocolError::FrameTooLarge(encoded.len()));
        }
        let frame: Self = rmp_serde::from_slice(encoded)?;
        if frame.version != TUNNEL_PROTOCOL_VERSION {
            return Err(TunnelProtocolError::UnsupportedVersion(frame.version));
        }
        Ok(frame)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelMessage {
    Welcome {
        public_url: String,
    },
    Replaced,
    RequestStart {
        request_id: Uuid,
        method: String,
        path_and_query: String,
        headers: Vec<TunnelHeader>,
        websocket: bool,
    },
    RequestBody {
        request_id: Uuid,
        #[serde(with = "serde_bytes")]
        bytes: Vec<u8>,
    },
    RequestEnd {
        request_id: Uuid,
    },
    RequestCancel {
        request_id: Uuid,
    },
    ResponseStart {
        request_id: Uuid,
        status: u16,
        headers: Vec<TunnelHeader>,
    },
    ResponseBody {
        request_id: Uuid,
        #[serde(with = "serde_bytes")]
        bytes: Vec<u8>,
    },
    ResponseEnd {
        request_id: Uuid,
    },
    ResponseError {
        request_id: Uuid,
        message: String,
    },
    WebSocketMessage {
        request_id: Uuid,
        message: TunnelWebSocketMessage,
    },
}

impl TunnelMessage {
    pub fn request_id(&self) -> Option<Uuid> {
        match self {
            Self::RequestStart { request_id, .. }
            | Self::RequestBody { request_id, .. }
            | Self::RequestEnd { request_id }
            | Self::RequestCancel { request_id }
            | Self::ResponseStart { request_id, .. }
            | Self::ResponseBody { request_id, .. }
            | Self::ResponseEnd { request_id }
            | Self::ResponseError { request_id, .. }
            | Self::WebSocketMessage { request_id, .. } => Some(*request_id),
            Self::Welcome { .. } | Self::Replaced => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelHeader {
    pub name: String,
    #[serde(with = "serde_bytes")]
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum TunnelWebSocketMessage {
    Text(String),
    Binary(#[serde(with = "serde_bytes")] Vec<u8>),
    Ping(#[serde(with = "serde_bytes")] Vec<u8>),
    Pong(#[serde(with = "serde_bytes")] Vec<u8>),
    Close { code: Option<u16>, reason: String },
}

#[derive(Debug, Error)]
pub enum TunnelProtocolError {
    #[error("tunnel frame is {0} bytes, exceeding the configured limit")]
    FrameTooLarge(usize),
    #[error("unsupported tunnel protocol version {0}")]
    UnsupportedVersion(u16),
    #[error("invalid tunnel frame: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("invalid tunnel frame: {0}")]
    Decode(#[from] rmp_serde::decode::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_round_trips_binary_payloads() {
        let request_id = Uuid::now_v7();
        let frame = TunnelFrame::new(TunnelMessage::ResponseBody {
            request_id,
            bytes: vec![0, 1, 2, 255],
        });

        assert_eq!(
            TunnelFrame::decode(&frame.encode().unwrap()).unwrap(),
            frame
        );
    }

    #[test]
    fn protocol_rejects_unknown_versions() {
        let encoded = rmp_serde::to_vec_named(&TunnelFrame {
            version: TUNNEL_PROTOCOL_VERSION + 1,
            message: TunnelMessage::Replaced,
        })
        .unwrap();

        assert!(matches!(
            TunnelFrame::decode(&encoded),
            Err(TunnelProtocolError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn protocol_round_trips_websocket_text_frames() {
        let frame = TunnelFrame::new(TunnelMessage::WebSocketMessage {
            request_id: Uuid::now_v7(),
            message: TunnelWebSocketMessage::Text("hello tunnel".to_owned()),
        });

        assert_eq!(
            TunnelFrame::decode(&frame.encode().unwrap()).unwrap(),
            frame
        );
    }
}
