//! The steps of `brume install`, in order. A new step goes here: it says whether it is done and
//! how to do it, and the install runs it only if it is not.

use std::rc::Rc;

use anyhow::bail;

use super::api_config::ApiConfig;
use super::cloudflare::{self, Cloudflare};
use super::preflight::Platform;
use super::wait::Console;
use super::{AdminMode, Options, Step, docker, nftables, systemd, tailscale, users};

/// What the steps need besides the options: the terminal and the clock of the steps that wait,
/// and Cloudflare, checked, with `--domain`.
pub struct Context {
    pub console: Rc<Console>,
    pub cloudflare: Option<Cloudflare>,
}

/// The steps for `options` on `platform`, in the order they run.
pub fn steps(
    options: &Options,
    platform: &Platform,
    context: &Context,
) -> anyhow::Result<Vec<Box<dyn Step>>> {
    if options.domain.is_some() != context.cloudflare.is_some() {
        bail!("--domain needs Cloudflare's checks first");
    }
    let mut steps: Vec<Box<dyn Step>> = vec![
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
    ];
    let api_config = match options.admin {
        AdminMode::SshTunnel => ApiConfig::SshTunnel,
        AdminMode::Tailscale => {
            // The admin's click comes early: the rest of the install runs after it.
            steps.extend::<[Box<dyn Step>; 5]>([
                Box::new(tailscale::Repository {
                    platform: platform.clone(),
                }),
                Box::new(tailscale::Package),
                Box::new(tailscale::Daemon),
                Box::new(tailscale::Up {
                    console: Rc::clone(&context.console),
                }),
                Box::new(tailscale::Serve {
                    console: Rc::clone(&context.console),
                }),
            ]);
            ApiConfig::Tailscale
        }
    };
    steps.extend::<[Box<dyn Step>; 5]>([
        // The agent's user joins the group `docker`, which Docker's package creates.
        Box::new(users::Users),
        Box::new(systemd::Directories),
        Box::new(api_config),
        Box::new(systemd::Units),
        Box::new(systemd::Services),
    ]);
    // The engine takes the tunnel once it runs.
    if let Some(cloudflare) = &context.cloudflare {
        steps.extend(cloudflare::steps(cloudflare, &context.console));
    }
    Ok(steps)
}
