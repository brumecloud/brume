//! The messages on `admin.sock`, which only local commands such as `brume dev` and `brume install`
//! use.

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::agent::serialize_exposed;

/// A request from a local command to the engine. Its `Debug` masks the tunnel token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdminRequest {
    /// A one-time login link for the admin.
    CreateLoginLink,
    /// The public entry, if it is set up: `brume install` checks it before setting it up again.
    GetIngress,
    /// Sets up the public entry: the zone whose subdomains the services get, and the Cloudflare
    /// tunnel that serves them. Sent again, it replaces the tunnel and its token.
    ConfigureIngress {
        /// A domain name such as `example.com`.
        zone: String,
        /// The Cloudflare tunnel's id.
        tunnel_id: String,
        /// The token that `cloudflared` runs the tunnel with. Serializes its exposed value:
        /// this message only travels on `admin.sock`.
        #[serde(serialize_with = "serialize_exposed")]
        tunnel_token: SecretString,
    },
}

impl PartialEq for AdminRequest {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CreateLoginLink, Self::CreateLoginLink)
            | (Self::GetIngress, Self::GetIngress) => true,
            (
                Self::ConfigureIngress {
                    zone,
                    tunnel_id,
                    tunnel_token,
                },
                Self::ConfigureIngress {
                    zone: other_zone,
                    tunnel_id: other_tunnel_id,
                    tunnel_token: other_tunnel_token,
                },
            ) => {
                zone == other_zone
                    && tunnel_id == other_tunnel_id
                    && tunnel_token.expose_secret() == other_tunnel_token.expose_secret()
            }
            _ => false,
        }
    }
}

impl Eq for AdminRequest {}

/// The engine's answer to an `AdminRequest`. Its `Debug` masks the token.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminReply {
    /// The link's token, to put in the panel's `/login#token=` URL.
    LoginLink {
        /// Base64url.
        token: String,
        /// RFC 3339.
        expires_at: String,
    },
    /// The public entry is set up.
    IngressConfigured,
    /// The public entry, or `None` before `brume install --domain`.
    Ingress(Option<AdminIngress>),
    /// The engine refused the request. `reason` says why, and never holds a secret of the
    /// request.
    Refused { reason: String },
}

/// The public entry, as the engine records it: no token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminIngress {
    /// A domain name such as `example.com`.
    pub zone: String,
    /// The Cloudflare tunnel's id.
    pub tunnel_id: String,
}

#[cfg(test)]
mod tests {
    use secrecy::{ExposeSecret, SecretString};

    use super::{AdminIngress, AdminReply, AdminRequest};

    const TOKEN: &str = "eyJhIjoiMSJ9-s3cr3t";

    fn configure(token: &str) -> AdminRequest {
        AdminRequest::ConfigureIngress {
            zone: "example.com".to_owned(),
            tunnel_id: "c1744f8b-faa1-48a4-9e5c-02ac921467fa".to_owned(),
            tunnel_token: SecretString::from(token),
        }
    }

    #[test]
    fn configure_ingress_round_trips_its_token() {
        let request = configure(TOKEN);
        let json = serde_json::to_string(&request).expect("serializes");
        assert!(json.contains(TOKEN), "{json}");
        match serde_json::from_str::<AdminRequest>(&json) {
            Ok(AdminRequest::ConfigureIngress { tunnel_token, .. }) => {
                assert_eq!(tunnel_token.expose_secret(), TOKEN);
            }
            other => panic!("expected ConfigureIngress, got {other:?}"),
        }
        assert_eq!(
            serde_json::from_str::<AdminRequest>(&json).ok(),
            Some(request)
        );
        assert_ne!(configure(TOKEN), configure("other"));
        assert_ne!(configure(TOKEN), AdminRequest::CreateLoginLink);
    }

    #[test]
    fn configure_ingress_debug_masks_the_token() {
        let debug = format!("{:?}", configure(TOKEN));
        assert!(!debug.contains(TOKEN), "{debug}");
        assert!(debug.contains("example.com"), "{debug}");
    }

    #[test]
    fn replies_round_trip() {
        for reply in [
            AdminReply::Ingress(None),
            AdminReply::Ingress(Some(AdminIngress {
                zone: "example.com".to_owned(),
                tunnel_id: "c1744f8b-faa1-48a4-9e5c-02ac921467fa".to_owned(),
            })),
            AdminReply::Refused {
                reason: "Another server writes to this bucket.".to_owned(),
            },
        ] {
            let json = serde_json::to_string(&reply).expect("serializes");
            assert_eq!(
                serde_json::from_str::<AdminReply>(&json).expect("parses"),
                reply
            );
        }
        assert_eq!(
            serde_json::to_string(&AdminRequest::GetIngress).expect("serializes"),
            "\"GetIngress\""
        );
    }
}
