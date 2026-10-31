//! The engine: the `World`, the log, and what the shell must do after each input.

use std::collections::{BTreeMap, BTreeSet};

use brume_domain::{
    AdminId, BlobId, Command, Ctx, DeploymentId, EventEnvelope, MachineId, Rejection,
    RejectionKind, SCHEMA_VERSION, Timestamp, World, decide, evolve,
};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_proto::agent::{AgentToEngine, EngineToAgent, ObservedContainer};
use brume_proto::api::{ApiFailure, ApiReply, ApiRequest, FailureKind};
use brume_proto::telemetry::LogQuery;
use sha2::{Digest, Sha256};

use crate::activity::History;
use crate::storage::{Storage, StorageError, StoredEvent, Transaction};

/// The log generation of this milestone: no compaction yet.
const GENERATION: u32 = 1;

/// The identity of the admin created at the first start.
const ADMIN_IDENTITY: &[u8] = b"admin";

/// A connection of the shell, so that a reply goes back where its request came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(pub u64);

/// What the shell feeds the engine.
#[derive(Debug)]
pub enum Input {
    /// A call from `brume api`.
    Api {
        client: ClientId,
        request: ApiRequest,
    },
    /// A request from a local command on `admin.sock`.
    Admin {
        client: ClientId,
        request: AdminRequest,
    },
    /// The agent of this machine connected.
    AgentConnected {
        machine: MachineId,
    },
    AgentDisconnected,
    FromAgent(AgentToEngine),
    /// The time of a `WakeAt` came, or the shell just started.
    Tick,
}

/// What the engine asks the shell to do.
#[derive(Debug)]
pub enum Output {
    Reply {
        client: ClientId,
        reply: ApiReply,
    },
    AdminReply {
        client: ClientId,
        reply: AdminReply,
    },
    ToAgent(EngineToAgent),
    /// The session is valid: the shell answers request `request` with `LogStreamOpened`, then
    /// relays telemetry's frames for `query` to the client, without interpreting them.
    OpenLogStream {
        client: ClientId,
        request: u64,
        query: LogQuery,
    },
    /// Feed `Input::Tick` at this instant. Replaces any earlier one.
    WakeAt(Timestamp),
}

/// Why `Engine::open` failed.
#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    /// The stored event at `seq` does not chain to the previous one.
    #[error("the event log's hash chain breaks at event {seq}")]
    BrokenChain { seq: u64 },
    /// The stored event at `seq` chains, but does not parse.
    #[error("event {seq} of the log does not parse")]
    Corrupt { seq: u64 },
    #[error("the first start failed: {0}")]
    Bootstrap(String),
}

/// Who caused an event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Actor {
    Admin(AdminId),
    Agent(MachineId),
    Engine,
}

impl Actor {
    fn as_string(&self) -> String {
        match self {
            Self::Admin(admin) => format!("admin:{admin}"),
            Self::Agent(machine) => format!("agent:{machine}"),
            Self::Engine => "engine".to_owned(),
        }
    }
}

/// The engine's state machine. One instance owns the log.
pub struct Engine<S: Storage> {
    pub(crate) storage: S,
    pub(crate) world: World,
    /// The `seq` of the last event.
    pub(crate) position: u64,
    pub(crate) last_hash: [u8; 32],
    /// The identity of each admin, read from its blob.
    pub(crate) admin_names: BTreeMap<AdminId, String>,
    /// What the activity view needs about entities the `World` forgot.
    pub(crate) history: History,
    /// The connected agent.
    pub(crate) agent: Option<MachineId>,
    /// The containers the agent last reported, by deployment id.
    pub(crate) observed: BTreeMap<String, ObservedContainer>,
    /// The position of the last desired state sent on this connection.
    pub(crate) sent_position: Option<u64>,
    /// The image resolutions asked on this connection.
    pub(crate) jobs_sent: BTreeSet<DeploymentId>,
    /// The last `WakeAt` sent.
    pub(crate) wake_at: Option<Timestamp>,
    /// Another server writes to the same bucket: every write is refused.
    pub(crate) read_only: bool,
}

impl<S: Storage> Engine<S> {
    /// Replays the log, checking its hash chain, and bootstraps the machine and its admin on
    /// an empty log.
    pub fn open(storage: S, ctx: &mut impl Ctx) -> Result<Self, OpenError> {
        let events = storage.load_events()?;
        let mut engine = Self {
            storage,
            world: World::new(),
            position: 0,
            last_hash: [0; 32],
            admin_names: BTreeMap::new(),
            history: History::default(),
            agent: None,
            observed: BTreeMap::new(),
            sent_position: None,
            jobs_sent: BTreeSet::new(),
            wake_at: None,
            read_only: false,
        };
        for stored in events {
            let seq = stored.seq;
            if seq != engine.position + 1 || chain(&engine.last_hash, &stored.bytes) != stored.hash
            {
                return Err(OpenError::BrokenChain { seq });
            }
            let envelope: EventEnvelope =
                serde_json::from_slice(&stored.bytes).map_err(|_| OpenError::Corrupt { seq })?;
            if envelope.seq != seq || envelope.prev_hash != prev_hash_hex(&engine.last_hash) {
                return Err(OpenError::BrokenChain { seq });
            }
            evolve(&mut engine.world, &envelope);
            engine.history.record(&envelope.event);
            engine.position = seq;
            engine.last_hash = stored.hash;
        }
        if engine.position == 0 {
            let identity = BlobId::new(ctx);
            let command = Command::Bootstrap {
                machine: MachineId::new(ctx),
                admin: AdminId::new(ctx),
                identity: identity.clone(),
            };
            let tx = Transaction {
                blobs: vec![(identity, ADMIN_IDENTITY.to_vec())],
                ..Transaction::default()
            };
            engine
                .commit(ctx, &Actor::Engine, &command, tx)
                .map_err(|failure| OpenError::Bootstrap(failure.message))?;
        }
        let admins = engine
            .world
            .admins()
            .map(|(admin, blob)| (admin.clone(), blob.clone()))
            .collect::<Vec<_>>();
        for (admin, blob) in admins {
            let name = engine
                .storage
                .blob(&blob)?
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| "admin".to_owned());
            engine.admin_names.insert(admin, name);
        }
        Ok(engine)
    }

    /// Handles one input, and pushes what the shell must do to `out`.
    pub fn handle(&mut self, ctx: &mut impl Ctx, input: Input, out: &mut Vec<Output>) {
        match input {
            Input::Api { client, request } => self.api(ctx, client, request, out),
            Input::Admin { client, request } => {
                let reply = match request {
                    AdminRequest::CreateLoginLink => self.create_login_link(ctx),
                };
                // A local command that gets no answer reports the closed connection.
                if let Ok(reply) = reply {
                    out.push(Output::AdminReply { client, reply });
                }
            }
            Input::AgentConnected { machine } => self.agent_connected(machine),
            Input::AgentDisconnected => self.agent_disconnected(),
            Input::FromAgent(message) => self.agent_message(ctx, message),
            Input::Tick => self.tick(ctx),
        }
        self.drive(ctx, out);
    }

    /// The `seq` of the last event of the log.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// The machine registered at the first start: the only agent the engine accepts.
    pub fn machine(&self) -> Option<&MachineId> {
        self.world.machine()
    }

    /// Refuses every write from now on: another server ships to the same bucket, and two
    /// writers would fork the log.
    pub fn set_read_only(&mut self) {
        self.read_only = true;
    }

    /// Whether `set_read_only` was called.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// The current state.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// The storage, for the shell's own reads.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Decides `command`, writes its events with `tx` in one transaction, then applies them.
    /// Nothing changes if the command is refused or the write fails.
    pub(crate) fn commit(
        &mut self,
        ctx: &mut impl Ctx,
        actor: &Actor,
        command: &Command,
        mut tx: Transaction,
    ) -> Result<Vec<EventEnvelope>, ApiFailure> {
        self.check_writable()?;
        let events = decide(&self.world, command).map_err(rejected)?;
        let at = ctx.now();
        let mut hash = self.last_hash;
        let mut seq = self.position;
        let mut envelopes = Vec::with_capacity(events.len());
        for event in events {
            seq += 1;
            let envelope = EventEnvelope {
                seq,
                generation: GENERATION,
                stream: event.stream(),
                schema_version: SCHEMA_VERSION,
                at,
                actor: actor.as_string(),
                correlation_id: None,
                causation_id: None,
                prev_hash: prev_hash_hex(&hash),
                event,
            };
            let bytes = serde_json::to_vec(&envelope).expect("an envelope serializes");
            hash = chain(&hash, &bytes);
            tx.events.push(StoredEvent { seq, bytes, hash });
            envelopes.push(envelope);
        }
        self.write(tx)?;
        for envelope in &envelopes {
            evolve(&mut self.world, envelope);
            self.history.record(&envelope.event);
        }
        self.position = seq;
        self.last_hash = hash;
        Ok(envelopes)
    }

    /// Writes a transaction without events, such as a session or a staged change.
    pub(crate) fn write(&mut self, tx: Transaction) -> Result<(), ApiFailure> {
        self.check_writable()?;
        let empty = tx.events.is_empty()
            && tx.blobs.is_empty()
            && tx.secrets.is_empty()
            && tx.staged_upserts.is_empty()
            && tx.staged_deletes.is_empty()
            && tx.sessions_upserts.is_empty()
            && tx.sessions_deletes.is_empty();
        if empty {
            return Ok(());
        }
        self.storage.commit(tx).map_err(storage_failure)
    }

    fn check_writable(&self) -> Result<(), ApiFailure> {
        if self.read_only {
            return Err(failure(
                FailureKind::Conflict,
                "Another server writes to this bucket.".to_owned(),
            ));
        }
        Ok(())
    }

    /// The admin's identity, for the panel.
    pub(crate) fn admin_name(&self, admin: &AdminId) -> String {
        self.admin_names
            .get(admin)
            .cloned()
            .unwrap_or_else(|| "admin".to_owned())
    }
}

/// `SHA-256(previous ‖ bytes)`.
pub(crate) fn chain(previous: &[u8; 32], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(previous);
    hasher.update(bytes);
    hasher.finalize().into()
}

/// The `prev_hash` of the event after the one with this hash: 64 zeros for the first event.
fn prev_hash_hex(hash: &[u8; 32]) -> String {
    hex::encode(hash)
}

/// An API failure.
pub(crate) fn failure(kind: FailureKind, message: String) -> ApiFailure {
    ApiFailure { kind, message }
}

/// A refused command, as an API failure.
pub(crate) fn rejected(rejection: Rejection) -> ApiFailure {
    let kind = match rejection.kind {
        RejectionKind::NotFound => FailureKind::NotFound,
        RejectionKind::Invalid => FailureKind::Invalid,
        RejectionKind::Conflict => FailureKind::Conflict,
    };
    failure(kind, rejection.message)
}

/// A storage failure, as an API failure. Its detail stays out of the reply.
pub(crate) fn storage_failure(_: StorageError) -> ApiFailure {
    failure(
        FailureKind::Internal,
        "The engine could not read or save its state.".to_owned(),
    )
}
