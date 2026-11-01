//! The steps of `brume install`, in order. A new step goes here: it says whether it is done and
//! how to do it, and the install runs it only if it is not.

use anyhow::bail;

use super::api_config::ApiConfig;
use super::preflight::Platform;
use super::{AdminMode, Options, Step, docker, nftables, systemd, users};

/// The steps for `options` on `platform`, in the order they run.
pub fn steps(options: &Options, platform: &Platform) -> anyhow::Result<Vec<Box<dyn Step>>> {
    if options.domain.is_some() {
        bail!(
            "--domain is not available yet. Run with --admin ssh-tunnel and without --domain: \
             public domains come in a later version."
        );
    }
    let api_config = match options.admin {
        AdminMode::SshTunnel => ApiConfig::ssh_tunnel(),
        AdminMode::Tailscale => bail!(
            "--admin tailscale, the default, is not available yet. Run with --admin ssh-tunnel: \
             the panel stays on 127.0.0.1 of the server, reached through an SSH tunnel."
        ),
    };
    Ok(vec![
        // The firewall comes first: no container ever runs without it.
        Box::new(nftables::NftablesPackage),
        Box::new(nftables::Firewall),
        // `daemon.json` is written before Docker's first start.
        Box::new(docker::Repository {
            platform: platform.clone(),
        }),
        Box::new(docker::Config),
        Box::new(docker::Packages),
        Box::new(docker::PinMajor),
        Box::new(docker::Running),
        // The agent's user joins the group `docker`, which Docker's package creates.
        Box::new(users::Users),
        Box::new(systemd::Directories),
        Box::new(api_config),
        Box::new(systemd::Units),
        Box::new(systemd::Services),
    ])
}
