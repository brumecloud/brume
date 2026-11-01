//! The public entry: `brume install` sets it up on `admin.sock`, the panel reads its zone.

use brume_domain::{Command, Ctx, Zone};
use brume_proto::admin::{AdminIngress, AdminReply};
use brume_proto::api::{ApiFailure, FailureKind, IngressDto};
use secrecy::{ExposeSecret, SecretString};

use crate::api::invalid;
use crate::engine::{Actor, Engine, failure};
use crate::storage::{Storage, Transaction};

/// The longest tunnel token, in bytes. Cloudflare's are a few hundred.
const MAX_TUNNEL_TOKEN_BYTES: usize = 4096;

impl<S: Storage> Engine<S> {
    /// Records the zone and the tunnel, and replaces the tunnel's token, sealed beside the log:
    /// the event never holds it. The new desired state carries the token to the agent.
    pub(crate) fn configure_ingress(
        &mut self,
        ctx: &mut impl Ctx,
        zone: &str,
        tunnel_id: String,
        tunnel_token: SecretString,
    ) -> Result<AdminReply, ApiFailure> {
        let zone = Zone::parse(zone).map_err(invalid)?;
        check_tunnel_token(&tunnel_token)?;
        let tx = Transaction {
            ingress_token: Some(tunnel_token),
            ..Transaction::default()
        };
        // A local command run by root on the host: no admin session carries it.
        self.commit(
            ctx,
            &Actor::Engine,
            &Command::ConfigureIngress { zone, tunnel_id },
            tx,
        )?;
        Ok(AdminReply::IngressConfigured)
    }

    /// The zone and the tunnel of the public entry, once set up, for `brume install`.
    pub(crate) fn admin_ingress(&self) -> Option<AdminIngress> {
        self.world.ingress().map(|ingress| AdminIngress {
            zone: ingress.zone.to_string(),
            tunnel_id: ingress.tunnel_id.clone(),
        })
    }

    /// The zone of the public entry, once set up.
    pub(crate) fn ingress_dto(&self) -> Option<IngressDto> {
        self.world.ingress().map(|ingress| IngressDto {
            zone: ingress.zone.to_string(),
        })
    }
}

/// 1 to 4,096 printable ASCII characters without spaces, as `cloudflared` reads it from its
/// environment. The message never repeats the token.
fn check_tunnel_token(token: &SecretString) -> Result<(), ApiFailure> {
    let token = token.expose_secret();
    if token.is_empty()
        || token.len() > MAX_TUNNEL_TOKEN_BYTES
        || !token.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(failure(
            FailureKind::Invalid,
            "Invalid tunnel token: must be 1 to 4,096 printable ASCII characters, without \
             spaces."
                .to_owned(),
        ));
    }
    Ok(())
}
