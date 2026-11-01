//! Who is at the other end of a unix socket, and who may be.
//!
//! With `brume dev`, every service runs as one user, the owner of the run directory: the sockets
//! are its own (`0600`). On a host, each service runs as its own system user: the sockets are
//! `0660`, their group shared by the four users, and each listener checks the uid of every peer
//! against its socket's role.

use std::io;
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _, PermissionsExt as _};
use std::path::Path;

use tokio::net::{UnixListener, UnixStream};

/// The system user of the engine on a host.
pub const ENGINE_USER: &str = "brume-engine";
/// The system user of the agent on a host, the only one in the `docker` group.
pub const AGENT_USER: &str = "brume-agent";
/// The system user of the API on a host.
pub const API_USER: &str = "brume-api";
/// The system user of telemetry on a host.
pub const TELEMETRY_USER: &str = "brume-telemetry";

/// Where the system users are read from. They are local users, created by `brume install`.
const PASSWD: &str = "/etc/passwd";

/// The uid of root, the only user besides the engine on `admin.sock` of a host.
const ROOT: u32 = 0;

/// The socket a connection arrives on. Each one has its own clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketRole {
    /// `api.sock`: `brume api` to the engine.
    Api,
    /// `agent.sock`: the agent to the engine.
    Agent,
    /// `admin.sock`: local commands to the engine, such as `brume login` run as root.
    Admin,
    /// `telemetry-ingest.sock`: the agent's container logs to telemetry.
    TelemetryIngest,
    /// `telemetry-query.sock`: the engine's log queries to telemetry.
    TelemetryQuery,
}

/// The users that the services accept on their sockets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peers {
    /// `brume dev`: every service runs as this user, the only one accepted on any socket.
    SameUser { uid: u32 },
    /// A host: each service runs as its own system user.
    System {
        engine: u32,
        agent: u32,
        api: u32,
        telemetry: u32,
    },
}

/// What a listener does with a new connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    Allowed,
    /// Another user: close the connection.
    Refused {
        uid: u32,
    },
    /// A peer gone before its credentials could be read, such as the readiness probe of
    /// `brume dev`, which connects and leaves at once: close the connection.
    Gone,
}

/// Why the system users cannot be resolved. The service does not start.
#[derive(Debug, thiserror::Error)]
pub enum PeersError {
    #[error("cannot read /etc/passwd")]
    Passwd(#[source] io::Error),
    #[error("the system user {0} is missing: `brume install` creates it")]
    MissingUser(&'static str),
    #[error("the system user {0} has an invalid uid in /etc/passwd")]
    InvalidUid(&'static str),
    #[error("the system user {0} is root: each service needs a user of its own")]
    Root(&'static str),
    #[error("the system users {0} and {1} share a uid: each service needs a user of its own")]
    SharedUid(&'static str, &'static str),
}

impl Peers {
    /// The peers of `brume dev`, as in jalon 1: the owner of `run_dir` only. Creates `run_dir`,
    /// readable by its owner only, if it is missing.
    pub fn same_user(run_dir: &Path) -> io::Result<Self> {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(run_dir)?;
        Ok(Self::SameUser {
            uid: owner_uid(run_dir)?,
        })
    }

    /// The peers of a host: the uid of each service's system user, read from `/etc/passwd`.
    pub fn system() -> Result<Self, PeersError> {
        let passwd = std::fs::read_to_string(PASSWD).map_err(PeersError::Passwd)?;
        Self::from_passwd(&passwd)
    }

    /// The peers of a host, from the contents of a `passwd` file. The first entry of a name
    /// wins, as for `getpwnam`. Each service needs a user of its own, which is not root.
    pub fn from_passwd(passwd: &str) -> Result<Self, PeersError> {
        let users = [ENGINE_USER, AGENT_USER, API_USER, TELEMETRY_USER];
        let mut uids = [0; 4];
        for (uid, name) in uids.iter_mut().zip(users) {
            *uid = uid_in_passwd(passwd, name)?;
            if *uid == ROOT {
                return Err(PeersError::Root(name));
            }
        }
        for (i, (uid, name)) in uids.iter().zip(users).enumerate() {
            if let Some((_, other)) = uids
                .iter()
                .zip(users)
                .skip(i + 1)
                .find(|(other_uid, _)| *other_uid == uid)
            {
                return Err(PeersError::SharedUid(name, other));
            }
        }
        let [engine, agent, api, telemetry] = uids;
        Ok(Self::System {
            engine,
            agent,
            api,
            telemetry,
        })
    }

    /// Whether the user `uid` may connect to `socket`.
    pub fn allows(&self, socket: SocketRole, uid: u32) -> bool {
        match *self {
            Self::SameUser { uid: owner } => uid == owner,
            Self::System {
                engine, agent, api, ..
            } => match socket {
                SocketRole::Api => uid == api,
                SocketRole::Agent | SocketRole::TelemetryIngest => uid == agent,
                SocketRole::Admin => uid == ROOT || uid == engine,
                SocketRole::TelemetryQuery => uid == engine,
            },
        }
    }

    /// What to do with a connection on `socket` whose peer is `uid`, as `peer_uid` reads it.
    pub fn admit(&self, socket: SocketRole, uid: io::Result<u32>) -> Admission {
        match uid {
            Ok(uid) if self.allows(socket, uid) => Admission::Allowed,
            Ok(uid) => Admission::Refused { uid },
            Err(_) => Admission::Gone,
        }
    }

    /// Listens on `path`, replacing a socket left by an earlier run: `0600` with `brume dev`,
    /// `0660` on a host, for the other services in the socket's group.
    pub fn listen(&self, path: &Path) -> io::Result<UnixListener> {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let listener = UnixListener::bind(path)?;
        let mode = match self {
            Self::SameUser { .. } => 0o600,
            Self::System { .. } => 0o660,
        };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
        Ok(listener)
    }
}

/// The uid of the first entry of `name` in `passwd`.
fn uid_in_passwd(passwd: &str, name: &'static str) -> Result<u32, PeersError> {
    let mut fields = passwd
        .lines()
        .map(|line| line.split(':'))
        .find_map(|mut fields| (fields.next() == Some(name)).then_some(fields))
        .ok_or(PeersError::MissingUser(name))?;
    // After the name: the password, then the uid.
    fields
        .nth(1)
        .and_then(|uid| uid.parse().ok())
        .ok_or(PeersError::InvalidUid(name))
}

/// The user id of the process at the other end of `stream`.
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    Ok(stream.peer_cred()?.uid())
}

/// The user id that owns `path`.
pub fn owner_uid(path: &Path) -> io::Result<u32> {
    Ok(std::fs::metadata(path)?.uid())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use super::{Admission, Peers, PeersError, SocketRole, owner_uid, peer_uid};

    const ENGINE: u32 = 990;
    const AGENT: u32 = 991;
    const API: u32 = 992;
    const TELEMETRY: u32 = 993;

    const SYSTEM: Peers = Peers::System {
        engine: ENGINE,
        agent: AGENT,
        api: API,
        telemetry: TELEMETRY,
    };

    const ROLES: [SocketRole; 5] = [
        SocketRole::Api,
        SocketRole::Agent,
        SocketRole::Admin,
        SocketRole::TelemetryIngest,
        SocketRole::TelemetryQuery,
    ];

    #[tokio::test]
    async fn peer_is_the_same_user_as_the_owner_of_its_files() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let (left, _right) = tokio::net::UnixStream::pair().expect("a socket pair");
        assert_eq!(
            peer_uid(&left).expect("credentials"),
            owner_uid(dir.path()).expect("metadata")
        );
    }

    #[test]
    fn peers_same_user_allows_owner_everywhere() {
        let peers = Peers::SameUser { uid: 501 };
        for role in ROLES {
            assert!(peers.allows(role, 501), "{role:?}");
            assert!(!peers.allows(role, 502), "{role:?}");
            assert!(!peers.allows(role, 0), "{role:?}");
        }
    }

    #[test]
    fn peers_system_refuses_api_uid_on_agent_socket() {
        assert!(!SYSTEM.allows(SocketRole::Agent, API));
        assert!(SYSTEM.allows(SocketRole::Agent, AGENT));
    }

    #[test]
    fn each_socket_of_a_host_has_its_own_clients() {
        let expected: [(SocketRole, &[u32]); 5] = [
            (SocketRole::Api, &[API]),
            (SocketRole::Agent, &[AGENT]),
            (SocketRole::Admin, &[0, ENGINE]),
            (SocketRole::TelemetryIngest, &[AGENT]),
            (SocketRole::TelemetryQuery, &[ENGINE]),
        ];
        for (role, allowed) in expected {
            for uid in [0, ENGINE, AGENT, API, TELEMETRY, 1000] {
                assert_eq!(
                    SYSTEM.allows(role, uid),
                    allowed.contains(&uid),
                    "{role:?} and uid {uid}"
                );
            }
        }
    }

    #[test]
    fn admission_follows_the_roles() {
        assert_eq!(SYSTEM.admit(SocketRole::Api, Ok(API)), Admission::Allowed);
        assert_eq!(
            SYSTEM.admit(SocketRole::Api, Ok(AGENT)),
            Admission::Refused { uid: AGENT }
        );
    }

    #[test]
    fn a_peer_gone_before_its_credentials_is_not_another_user() {
        let gone = std::io::Error::from(std::io::ErrorKind::NotConnected);
        assert_eq!(SYSTEM.admit(SocketRole::Api, Err(gone)), Admission::Gone);
    }

    const PASSWD: &str = "\
root:x:0:0:root:/root:/bin/bash
# a comment

brume-engine:x:990:990::/nonexistent:/usr/sbin/nologin
brume-agent:x:991:991::/nonexistent:/usr/sbin/nologin
brume-api:x:992:992::/nonexistent:/usr/sbin/nologin
brume-telemetry:x:993:993::/nonexistent:/usr/sbin/nologin
brume-engine:x:1500:1500::/nonexistent:/usr/sbin/nologin
";

    #[test]
    fn resolves_the_system_users_from_passwd() {
        // The first entry of a name wins, as for getpwnam.
        assert_eq!(Peers::from_passwd(PASSWD).expect("resolves"), SYSTEM);
    }

    #[test]
    fn names_a_missing_system_user() {
        let passwd = PASSWD.replace("brume-api:", "brume-apx:");
        let error = Peers::from_passwd(&passwd).expect_err("brume-api is missing");
        assert!(
            matches!(error, PeersError::MissingUser("brume-api")),
            "{error:?}"
        );
        assert!(error.to_string().contains("brume-api"), "{error}");
    }

    #[test]
    fn refuses_root_or_a_shared_user() {
        let root = PASSWD.replace("brume-agent:x:991", "brume-agent:x:0");
        assert!(matches!(
            Peers::from_passwd(&root),
            Err(PeersError::Root("brume-agent"))
        ));
        let shared = PASSWD.replace("brume-api:x:992", "brume-api:x:991");
        assert!(matches!(
            Peers::from_passwd(&shared),
            Err(PeersError::SharedUid("brume-agent", "brume-api"))
        ));
        let invalid = PASSWD.replace("brume-telemetry:x:993", "brume-telemetry:x:-1");
        assert!(matches!(
            Peers::from_passwd(&invalid),
            Err(PeersError::InvalidUid("brume-telemetry"))
        ));
    }

    #[tokio::test]
    async fn sockets_are_private_with_brume_dev_and_shared_on_a_host() {
        let dir = tempfile::Builder::new()
            .prefix("brume-peer")
            .tempdir_in("/tmp")
            .expect("a temporary directory");
        let peers = Peers::same_user(&dir.path().join("run")).expect("creates the run directory");
        let run_mode = std::fs::metadata(dir.path().join("run"))
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(run_mode & 0o777, 0o700);
        for (name, peers, mode) in [("dev.sock", peers, 0o600), ("host.sock", SYSTEM, 0o660)] {
            let socket = dir.path().join("run").join(name);
            std::fs::write(&socket, b"stale").expect("a stale file");
            let _listener = peers.listen(&socket).expect("listens");
            let metadata = std::fs::metadata(&socket).expect("metadata");
            assert_eq!(metadata.permissions().mode() & 0o777, mode, "{peers:?}");
            assert!(std::os::unix::fs::FileTypeExt::is_socket(
                &metadata.file_type()
            ));
        }
    }
}
