//! The system users of the services: one each, all in the group `brume`, which owns the
//! sockets' directory. Only the agent joins `docker`.

use brume_runtime::peer::AGENT_USER;

use super::Step;
use super::host::{Cmd, Host};
use super::systemd::Service;

/// The group of the four services, the group of `/run/brume` and of every socket in it.
pub const SHARED_GROUP: &str = "brume";

/// The group of Docker's socket, created by Docker's package.
const DOCKER_GROUP: &str = "docker";

/// The groups that `user` needs, besides its own.
fn groups(user: &str) -> &'static [&'static str] {
    if user == AGENT_USER {
        &[SHARED_GROUP, DOCKER_GROUP]
    } else {
        &[SHARED_GROUP]
    }
}

pub fn group_query(group: &str) -> Cmd {
    Cmd::new("getent", &["group", group])
}

pub fn user_query(user: &str) -> Cmd {
    Cmd::new("getent", &["passwd", user])
}

/// Prints the names of the groups of `user`.
pub fn groups_query(user: &str) -> Cmd {
    Cmd::new("id", &["--groups", "--name", user])
}

/// The groups that `user` is missing, or `None` if `user` is missing.
fn missing_groups(host: &Host, user: &str) -> anyhow::Result<Option<Vec<&'static str>>> {
    if !host.query(&user_query(user))?.success() {
        return Ok(None);
    }
    let current = host.query_stdout(&groups_query(user))?.unwrap_or_default();
    let current: Vec<&str> = current.split_whitespace().collect();
    Ok(Some(
        groups(user)
            .iter()
            .copied()
            .filter(|group| !current.contains(group))
            .collect(),
    ))
}

/// Creates the group `brume` and the users of the services.
pub struct Users;

impl Step for Users {
    fn name(&self) -> &str {
        "Create the system users of the services"
    }

    fn is_done(&self, host: &Host) -> anyhow::Result<bool> {
        if !host.query(&group_query(SHARED_GROUP))?.success() {
            return Ok(false);
        }
        for service in Service::ALL {
            if missing_groups(host, service.user())? != Some(Vec::new()) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn apply(&self, host: &mut Host) -> anyhow::Result<()> {
        if !host.query(&group_query(SHARED_GROUP))?.success() {
            host.change(&Cmd::new("groupadd", &["--system", SHARED_GROUP]))?;
        }
        for service in Service::ALL {
            let user = service.user();
            let wanted = groups(user).join(",");
            match missing_groups(host, user)? {
                None => {
                    host.change(&Cmd::new(
                        "useradd",
                        &[
                            "--system",
                            "--user-group",
                            "--no-create-home",
                            "--home-dir",
                            "/nonexistent",
                            "--shell",
                            "/usr/sbin/nologin",
                            "--groups",
                            &wanted,
                            user,
                        ],
                    ))?;
                }
                Some(missing) if !missing.is_empty() => {
                    host.change(&Cmd::new(
                        "usermod",
                        &["--append", "--groups", &missing.join(","), user],
                    ))?;
                }
                Some(_) => {}
            }
        }
        Ok(())
    }
}
