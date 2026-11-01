//! Login links and sessions. Tokens never leave the engine except in the reply that creates
//! them; storage only holds their SHA-256.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use brume_domain::{AuditEvent, Command, Ctx};
use brume_proto::admin::AdminReply;
use brume_proto::api::{ApiFailure, ApiValue, FailureKind};
use sha2::{Digest, Sha256};

use crate::engine::{Actor, Engine, failure, storage_failure};
use crate::storage::{SessionKind, Storage, StoredSession, Transaction};

/// How long a login link stays valid: 15 minutes.
pub const LOGIN_LINK_MILLIS: u64 = 15 * 60 * 1000;

/// How long a session stays valid: 12 hours.
pub const SESSION_MILLIS: u64 = 12 * 60 * 60 * 1000;

/// The SHA-256 of a token, as storage keys it.
pub fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// 32 random bytes in base64url, without padding.
fn new_token(ctx: &mut impl Ctx) -> String {
    let mut bytes = [0; 32];
    ctx.fill_random(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn unauthorized() -> ApiFailure {
    failure(FailureKind::Unauthorized, "Log in to continue.".to_owned())
}

impl<S: Storage> Engine<S> {
    /// A one-time login link for the admin, valid for 15 minutes.
    pub(crate) fn create_login_link(
        &mut self,
        ctx: &mut impl Ctx,
    ) -> Result<AdminReply, ApiFailure> {
        let Some(admin) = self.world.admins().next().map(|(admin, _)| admin.clone()) else {
            return Err(failure(
                FailureKind::Internal,
                "This server has no admin.".to_owned(),
            ));
        };
        let token = new_token(ctx);
        let expires_at = ctx.now().plus_millis(LOGIN_LINK_MILLIS);
        self.write(Transaction {
            sessions_upserts: vec![StoredSession {
                token_hash: token_hash(&token),
                admin,
                kind: SessionKind::LoginLink,
                expires_at,
            }],
            ..Transaction::default()
        })?;
        Ok(AdminReply::LoginLink {
            token,
            expires_at: expires_at.to_rfc3339(),
        })
    }

    /// The session of a request's token, if it is valid.
    pub(crate) fn authenticate(
        &self,
        ctx: &impl Ctx,
        token: Option<&str>,
    ) -> Result<StoredSession, ApiFailure> {
        let Some(token) = token else {
            return Err(unauthorized());
        };
        match self
            .storage
            .session(&token_hash(token))
            .map_err(storage_failure)?
        {
            Some(session)
                if session.kind == SessionKind::Session && session.expires_at > ctx.now() =>
            {
                Ok(session)
            }
            _ => Err(unauthorized()),
        }
    }

    /// Consumes a login link and opens a 12-hour session. Records `LoginSucceeded` or
    /// `LoginFailed`.
    pub(crate) fn login(
        &mut self,
        ctx: &mut impl Ctx,
        token: &str,
    ) -> Result<ApiValue, ApiFailure> {
        let link_hash = token_hash(token);
        let link = self.storage.session(&link_hash).map_err(storage_failure)?;
        let now = ctx.now();
        match link {
            Some(link) if link.kind == SessionKind::LoginLink && link.expires_at > now => {
                let session = new_token(ctx);
                let tx = Transaction {
                    sessions_deletes: vec![link_hash],
                    sessions_upserts: vec![StoredSession {
                        token_hash: token_hash(&session),
                        admin: link.admin.clone(),
                        kind: SessionKind::Session,
                        expires_at: now.plus_millis(SESSION_MILLIS),
                    }],
                    ..Transaction::default()
                };
                let command = Command::RecordAudit(AuditEvent::LoginSucceeded {
                    admin: link.admin.clone(),
                });
                self.commit(ctx, &Actor::Admin(link.admin.clone()), &command, tx)?;
                Ok(ApiValue::Session {
                    token: session,
                    admin: self.admin_name(&link.admin),
                })
            }
            other => {
                // An expired link goes too; a session token is not a login link.
                let sessions_deletes = match other {
                    Some(link) if link.kind == SessionKind::LoginLink => vec![link_hash],
                    _ => Vec::new(),
                };
                let tx = Transaction {
                    sessions_deletes,
                    ..Transaction::default()
                };
                let command = Command::RecordAudit(AuditEvent::LoginFailed);
                self.commit(ctx, &Actor::Engine, &command, tx)?;
                Err(failure(
                    FailureKind::Unauthorized,
                    "This login link is invalid, expired or already used.".to_owned(),
                ))
            }
        }
    }

    /// Closes a session. Records `SessionClosed`.
    pub(crate) fn logout(
        &mut self,
        ctx: &mut impl Ctx,
        session: &StoredSession,
    ) -> Result<ApiValue, ApiFailure> {
        let tx = Transaction {
            sessions_deletes: vec![session.token_hash],
            ..Transaction::default()
        };
        let command = Command::RecordAudit(AuditEvent::SessionClosed {
            admin: session.admin.clone(),
        });
        self.commit(ctx, &Actor::Admin(session.admin.clone()), &command, tx)?;
        Ok(ApiValue::NoContent)
    }
}
