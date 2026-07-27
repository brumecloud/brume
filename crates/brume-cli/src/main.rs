mod archive;
mod config;
mod embedded;
mod output;
mod preview;
mod progress;
mod renderer;
mod tunnel;

mod build_metadata {
    include!(concat!(env!("OUT_DIR"), "/build_metadata.rs"));
}

use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use brume_api_client::{BrumeClient, DeploySiteOptions};
use brume_core::{AuthMode, DeploymentPatch, PlanPatch, PollCliLoginResponse};
use clap::{Parser, Subcommand};
use output::OutputFormat;
use serde_json::json;
use tempfile::TempDir;

#[derive(Parser)]
#[command(
    name = "brume",
    version,
    about = "Publish agent plans and static HTML sites"
)]
struct Cli {
    #[arg(long, env = "BRUME_BASE_URL", default_value = "https://api.brume.dev")]
    base_url: String,
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t,
        value_name = "FORMAT",
        help = "Select human-readable or JSON output"
    )]
    output: OutputFormat,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Version,
    Login,
    Deploy {
        #[command(subcommand)]
        command: Option<DeployCommand>,
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        spa: bool,
        #[arg(long)]
        pin: bool,
        #[arg(long, default_value_t)]
        auth: AuthMode,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long)]
        no_overlay: bool,
    },
    Tunnel {
        port: u16,
        #[arg(long)]
        url: Option<String>,
    },
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
}

#[derive(Subcommand)]
enum McpCommand {
    Serve,
    Config,
}

#[derive(Subcommand)]
enum DeployCommand {
    List,
    Show {
        deployment: String,
    },
    Auth {
        deployment: String,
        auth: AuthMode,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long)]
        overlay: Option<bool>,
    },
    Delete {
        deployment: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum PlanCommand {
    Preview {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long, default_value_t = 0)]
        port: u16,
        #[arg(long)]
        no_open: bool,
    },
    Build {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long, alias = "output-dir")]
        destination: Option<PathBuf>,
    },
    Deploy {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        slug: Option<String>,
        #[arg(long)]
        auth: Option<AuthMode>,
        #[arg(long)]
        pin: bool,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long)]
        no_overlay: bool,
    },
    List,
    Show {
        plan: String,
    },
    Open {
        plan: String,
    },
    Auth {
        plan: String,
        auth: AuthMode,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long)]
        overlay: Option<bool>,
    },
    Pin {
        plan: String,
    },
    Unpin {
        plan: String,
    },
    Delete {
        plan: String,
        #[arg(long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "brume=info".into()),
        )
        .with_writer(io::stderr)
        .init();
    let cli = Cli::parse();
    match cli.command {
        Command::Version => {
            let short_commit = build_metadata::COMMIT_SHA
                .chars()
                .take(7)
                .collect::<String>();
            if cli.output.is_json() {
                output::json(&json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "commit": {
                        "sha": build_metadata::COMMIT_SHA,
                        "short_sha": short_commit,
                        "title": build_metadata::COMMIT_TITLE,
                        "message": build_metadata::COMMIT_MESSAGE,
                    }
                }))?;
            } else {
                println!("brume {} {}", env!("CARGO_PKG_VERSION"), short_commit);
                println!("commit title: {}", build_metadata::COMMIT_TITLE);
                println!("commit message: {}", build_metadata::COMMIT_MESSAGE);
            }
            Ok(())
        }
        Command::Login => login(&cli.base_url, cli.output).await,
        Command::Deploy {
            command,
            directory,
            url,
            spa,
            pin,
            auth,
            password_stdin,
            no_overlay,
        } => match command {
            Some(command) => deploy_command(&cli.base_url, command, cli.output).await,
            None => {
                deploy(
                    &cli.base_url,
                    DeployOptions {
                        directory: &directory,
                        slug: url,
                        spa,
                        pinned: pin,
                        auth,
                        password_stdin,
                        overlay_enabled: !no_overlay,
                    },
                    cli.output,
                )
                .await
            }
        },
        Command::Tunnel { port, url } => {
            if let Some(url) = &url {
                config::validate_slug(url)?;
            }
            tunnel::run(&cli.base_url, port, url.as_deref(), cli.output).await
        }
        Command::Mcp { command } => mcp(&cli.base_url, command, cli.output).await,
        Command::Plan { command } => plan(&cli.base_url, command, cli.output).await,
    }
}

async fn deploy_command(
    base_url: &str,
    command: DeployCommand,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        DeployCommand::List => {
            let mut progress = progress::Progress::start(2, "Authenticating");
            let client = authenticated_client(base_url).await?;
            progress.advance("Loading deployments");
            let response = client.list_deployments().await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&response)?;
            } else {
                output::deployments(&response)?;
            }
            Ok(())
        }
        DeployCommand::Show { deployment } => {
            let mut progress = progress::Progress::start(2, "Authenticating");
            let client = authenticated_client(base_url).await?;
            progress.advance(format!("Loading deployment {deployment}"));
            let response = client.get_deployment(&deployment).await?;
            progress.finish();
            output::json(&response)
        }
        DeployCommand::Auth {
            deployment,
            auth,
            password_stdin,
            overlay,
        } => {
            let mut progress = progress::Progress::start(2, "Authenticating");
            let client = authenticated_client(base_url).await?;
            let password = deployment_password(auth, password_stdin)?;
            progress.advance(format!("Changing authentication for {deployment}"));
            let response = client
                .patch_deployment(
                    &deployment,
                    &DeploymentPatch {
                        auth: Some(auth),
                        password,
                        overlay_enabled: overlay,
                    },
                )
                .await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&response)?;
            } else {
                println!("{} is now {}", response.slug, response.auth);
            }
            Ok(())
        }
        DeployCommand::Delete { deployment, yes } => {
            delete_deployment(base_url, &deployment, yes, output_format).await
        }
    }
}

struct DeployOptions<'a> {
    directory: &'a Path,
    slug: Option<String>,
    spa: bool,
    pinned: bool,
    auth: AuthMode,
    password_stdin: bool,
    overlay_enabled: bool,
}

async fn deploy(
    base_url: &str,
    options: DeployOptions<'_>,
    output_format: OutputFormat,
) -> Result<()> {
    let DeployOptions {
        directory,
        slug,
        spa,
        pinned,
        auth,
        password_stdin,
        overlay_enabled,
    } = options;
    let mut progress = progress::Progress::start(4, "Authenticating");
    let token = config::load_access_token(base_url).await?;
    progress.advance("Validating deployment");
    let source = canonical_directory(directory)?;
    if let Some(slug) = &slug {
        config::validate_slug(slug)?;
    }
    progress.advance("Packaging files");
    let archive = archive::create_deployment_archive(&source)?;
    let password = deployment_password(auth, password_stdin)?;
    progress.advance(format!(
        "Uploading and deploying {}",
        human_bytes(archive.len())
    ));
    let deployed = BrumeClient::new(base_url, Some(token))?
        .deploy_site(DeploySiteOptions {
            slug: slug.as_deref(),
            spa,
            auth,
            password: password.as_deref(),
            overlay_enabled,
            pinned,
            archive,
        })
        .await?;
    progress.finish();
    if output_format.is_json() {
        output::json(&deployed)?;
    } else {
        println!("Deployed {}", deployed.deployment.url);
    }
    Ok(())
}

async fn delete_deployment(
    base_url: &str,
    deployment: &str,
    yes: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let mut progress = progress::Progress::start(2, "Authenticating");
    let client = authenticated_client(base_url).await?;
    progress.advance(format!("Preparing deletion for {deployment}"));
    let challenge = client
        .create_deployment_deletion_challenge(deployment)
        .await?;
    progress.finish();
    if !yes {
        if output_format.is_json() {
            eprint!(
                "Delete `{}` and all of its files permanently? [y/N] ",
                challenge.deployment.slug
            );
            io::stderr().flush()?;
        } else {
            print!(
                "Delete `{}` and all of its files permanently? [y/N] ",
                challenge.deployment.slug
            );
            io::stdout().flush()?;
        }
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
            if output_format.is_json() {
                output::json(&json!({
                    "status": "cancelled",
                    "deployment": challenge.deployment,
                }))?;
            } else {
                println!("Deletion cancelled");
            }
            return Ok(());
        }
    }
    let progress = progress::Progress::start(1, format!("Deleting deployment {deployment}"));
    client
        .confirm_deployment_deletion(deployment, challenge.challenge)
        .await?;
    progress.finish();
    if output_format.is_json() {
        output::json(&json!({
            "status": "deleted",
            "deployment": challenge.deployment,
        }))?;
    } else {
        println!("Deleted {}", challenge.deployment.slug);
    }
    Ok(())
}

async fn login(base_url: &str, output_format: OutputFormat) -> Result<()> {
    let client = BrumeClient::new(base_url, None)?;
    let session = client.begin_cli_login().await?;
    if !output_format.is_json() {
        println!("Opening {}", session.browser_url);
    }
    open::that(&session.browser_url).context("opening GitHub login in the default browser")?;
    let deadline = Instant::now() + Duration::from_secs(session.expires_in_seconds);
    loop {
        if Instant::now() >= deadline {
            bail!("login session expired; run `brume login` again");
        }
        match client
            .poll_cli_login(&session.session_id.to_string(), &session.poll_secret)
            .await?
        {
            PollCliLoginResponse::Pending => tokio::time::sleep(Duration::from_secs(2)).await,
            PollCliLoginResponse::Authorized {
                credentials,
                user_handle,
            } => {
                config::save_credentials(base_url, &credentials)?;
                if output_format.is_json() {
                    output::json(&json!({
                        "status": "authorized",
                        "user_handle": user_handle,
                    }))?;
                } else {
                    println!("Logged in as @{user_handle}");
                }
                return Ok(());
            }
            PollCliLoginResponse::Expired => {
                bail!("login session expired; run `brume login` again")
            }
        }
    }
}

async fn mcp(base_url: &str, command: McpCommand, output_format: OutputFormat) -> Result<()> {
    match command {
        McpCommand::Serve => {
            let _ = config::load_access_token(base_url).await?;
            let credential_url = base_url.to_owned();
            let token_loader: brume_mcp::TokenLoader = Arc::new(move || {
                let credential_url = credential_url.clone();
                Box::pin(async move {
                    config::load_access_token(&credential_url)
                        .await
                        .map_err(|error| error.to_string())
                })
            });
            brume_mcp::serve(base_url, token_loader).await
        }
        McpCommand::Config => {
            if output_format.is_json() {
                output::json(&json!({
                    "mcp_servers": {
                        "brume": {
                            "command": "brume",
                            "args": ["--base-url", base_url, "mcp", "serve"]
                        }
                    }
                }))?;
            } else {
                println!(
                    "[mcp_servers.brume]\ncommand = \"brume\"\nargs = [\"--base-url\", \"{base_url}\", \"mcp\", \"serve\"]"
                );
            }
            Ok(())
        }
    }
}

async fn plan(base_url: &str, command: PlanCommand, output_format: OutputFormat) -> Result<()> {
    match command {
        PlanCommand::Preview {
            directory,
            port,
            no_open,
        } => {
            let mut progress = progress::Progress::start(3, "Loading plan project");
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let temporary = TempDir::new()?;
            progress.advance("Rendering plan");
            let rendered = renderer::render(
                &source,
                temporary.path(),
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
            progress.advance("Starting preview server");
            progress.finish();
            preview::serve(
                temporary.path().to_path_buf(),
                rendered.manifest,
                port,
                !no_open,
                rendered.page_count,
                rendered.asset_count,
                output_format,
            )
            .await
        }
        PlanCommand::Build {
            directory,
            destination,
        } => {
            let mut progress = progress::Progress::start(2, "Loading plan project");
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let destination =
                absolute_path(destination.unwrap_or_else(|| source.join(".brume").join("dist")))?;
            progress.advance("Rendering plan");
            let rendered = renderer::render(
                &source,
                &destination,
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&json!({
                    "page_count": rendered.page_count,
                    "asset_count": rendered.asset_count,
                    "destination": destination,
                }))?;
            } else {
                println!(
                    "Built {} pages and {} assets in {}",
                    rendered.page_count,
                    rendered.asset_count,
                    destination.display()
                );
            }
            Ok(())
        }
        PlanCommand::Deploy {
            directory,
            slug,
            auth,
            pin,
            password_stdin,
            no_overlay,
        } => {
            let mut progress = progress::Progress::start(5, "Authenticating");
            let token = config::load_access_token(base_url).await?;
            progress.advance("Loading plan project");
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let slug = slug
                .or(project.plan.slug.clone())
                .map(Ok)
                .unwrap_or_else(|| config::default_slug(&source))?;
            config::validate_slug(&slug)?;
            let auth = auth.or(project.plan.auth).unwrap_or_default();
            let overlay_enabled = if no_overlay {
                false
            } else {
                project.plan.overlay.unwrap_or(true)
            };
            let password = deployment_password(auth, password_stdin)?;
            let temporary = TempDir::new()?;
            progress.advance("Rendering plan");
            renderer::render(
                &source,
                temporary.path(),
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
            progress.advance("Packaging rendered plan");
            let archive = archive::create_bundle_archive(temporary.path())?;
            progress.advance(format!(
                "Uploading and deploying {}",
                human_bytes(archive.len())
            ));
            let deployed = BrumeClient::new(base_url, Some(token))?
                .deploy(
                    &slug,
                    auth,
                    password.as_deref(),
                    overlay_enabled,
                    pin,
                    archive,
                )
                .await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&deployed)?;
            } else {
                println!("Deployed {}", deployed.plan.summary.url);
            }
            Ok(())
        }
        PlanCommand::List => {
            let mut progress = progress::Progress::start(2, "Authenticating");
            let client = authenticated_client(base_url).await?;
            progress.advance("Loading plans");
            let response = client.list_plans().await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&response)?;
            } else {
                output::plans(&response)?;
            }
            Ok(())
        }
        PlanCommand::Show { plan } => {
            let mut progress = progress::Progress::start(2, "Authenticating");
            let client = authenticated_client(base_url).await?;
            progress.advance(format!("Loading plan {plan}"));
            let details = client.get_plan(&plan).await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&details)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&details)?);
            }
            Ok(())
        }
        PlanCommand::Open { plan } => {
            let mut progress = progress::Progress::start(3, "Authenticating");
            let client = authenticated_client(base_url).await?;
            progress.advance(format!("Loading plan {plan}"));
            let details = client.get_plan(&plan).await?;
            progress.advance("Opening plan in browser");
            open::that(&details.summary.url)?;
            progress.finish();
            if output_format.is_json() {
                output::json(&details)?;
            }
            Ok(())
        }
        PlanCommand::Auth {
            plan,
            auth,
            password_stdin,
            overlay,
        } => {
            let mut progress = progress::Progress::start(2, "Authenticating");
            let client = authenticated_client(base_url).await?;
            let password = deployment_password(auth, password_stdin)?;
            progress.advance(format!("Changing authentication for {plan}"));
            let details = client
                .patch_plan(
                    &plan,
                    &PlanPatch {
                        auth: Some(auth),
                        password,
                        overlay_enabled: overlay,
                        pinned: None,
                    },
                )
                .await?;
            progress.finish();
            if output_format.is_json() {
                output::json(&details)?;
            } else {
                println!("{} is now {}", details.summary.slug, details.summary.auth);
            }
            Ok(())
        }
        PlanCommand::Pin { plan } => patch_pin(base_url, &plan, true, output_format).await,
        PlanCommand::Unpin { plan } => patch_pin(base_url, &plan, false, output_format).await,
        PlanCommand::Delete { plan, yes } => delete_plan(base_url, &plan, yes, output_format).await,
    }
}

async fn patch_pin(
    base_url: &str,
    plan: &str,
    pinned: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let mut progress = progress::Progress::start(2, "Authenticating");
    let client = authenticated_client(base_url).await?;
    progress.advance(if pinned {
        format!("Pinning plan {plan}")
    } else {
        format!("Unpinning plan {plan}")
    });
    let details = client
        .patch_plan(
            plan,
            &PlanPatch {
                auth: None,
                password: None,
                overlay_enabled: None,
                pinned: Some(pinned),
            },
        )
        .await?;
    progress.finish();
    if output_format.is_json() {
        output::json(&details)?;
    } else {
        println!(
            "{} is {}",
            details.summary.slug,
            if pinned {
                "pinned"
            } else {
                "subject to retention"
            }
        );
    }
    Ok(())
}

async fn delete_plan(
    base_url: &str,
    plan: &str,
    yes: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let mut progress = progress::Progress::start(2, "Authenticating");
    let client = authenticated_client(base_url).await?;
    progress.advance(format!("Preparing deletion for {plan}"));
    let challenge = client.create_deletion_challenge(plan).await?;
    progress.finish();
    if !yes {
        if output_format.is_json() {
            eprint!(
                "Delete `{}` and all of its files permanently? [y/N] ",
                challenge.plan.slug
            );
            io::stderr().flush()?;
        } else {
            print!(
                "Delete `{}` and all of its files permanently? [y/N] ",
                challenge.plan.slug
            );
            io::stdout().flush()?;
        }
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
            if output_format.is_json() {
                output::json(&json!({
                    "status": "cancelled",
                    "plan": challenge.plan,
                }))?;
            } else {
                println!("Deletion cancelled");
            }
            return Ok(());
        }
    }
    let progress = progress::Progress::start(1, format!("Deleting plan {plan}"));
    client.confirm_deletion(plan, challenge.challenge).await?;
    progress.finish();
    if output_format.is_json() {
        output::json(&json!({
            "status": "deleted",
            "plan": challenge.plan,
        }))?;
    } else {
        println!("Deleted {}", challenge.plan.slug);
    }
    Ok(())
}

async fn authenticated_client(base_url: &str) -> Result<BrumeClient> {
    BrumeClient::new(base_url, Some(config::load_access_token(base_url).await?)).map_err(Into::into)
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(path)
        .with_context(|| format!("opening plan directory {}", path.display()))?;
    if !path.is_dir() {
        bail!("{} is not a directory", path.display());
    }
    Ok(path)
}

fn absolute_path(path: PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn human_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    let bytes = bytes as f64;
    if bytes >= MIB {
        format!("{:.1} MiB", bytes / MIB)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes / KIB)
    } else {
        format!("{bytes:.0} B")
    }
}

fn deployment_password(auth: AuthMode, password_stdin: bool) -> Result<Option<String>> {
    if auth != AuthMode::Password {
        if password_stdin {
            bail!("--password-stdin requires --auth password");
        }
        return Ok(None);
    }
    let password = if password_stdin {
        let mut password = String::new();
        io::stdin()
            .read_to_string(&mut password)
            .context("reading the deployment password from stdin")?;
        password
            .strip_suffix("\r\n")
            .or_else(|| password.strip_suffix('\n'))
            .unwrap_or(&password)
            .to_owned()
    } else {
        if !io::stdin().is_terminal() {
            bail!("password authentication requires an interactive terminal or --password-stdin");
        }
        let password =
            rpassword::prompt_password("Website password: ").context("reading website password")?;
        let confirmation = rpassword::prompt_password("Confirm password: ")
            .context("reading website password confirmation")?;
        if password != confirmation {
            bail!("website password confirmation does not match");
        }
        password
    };
    if !(8..=256).contains(&password.chars().count()) {
        bail!("website password must contain between 8 and 256 characters");
    }
    Ok(Some(password))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_is_accepted_after_nested_commands() {
        let cli = Cli::try_parse_from(["brume", "plan", "list", "--output", "json"])
            .expect("global output option should parse after nested commands");

        assert_eq!(cli.output, OutputFormat::Json);
        assert!(matches!(
            cli.command,
            Command::Plan {
                command: PlanCommand::List
            }
        ));
    }

    #[test]
    fn build_destination_does_not_conflict_with_output_format() {
        let cli = Cli::try_parse_from([
            "brume",
            "plan",
            "build",
            ".",
            "--destination",
            "./dist",
            "--output",
            "json",
        ])
        .expect("build destination and output format should parse together");

        assert_eq!(cli.output, OutputFormat::Json);
        match cli.command {
            Command::Plan {
                command:
                    PlanCommand::Build {
                        directory,
                        destination,
                    },
            } => {
                assert_eq!(directory, PathBuf::from("."));
                assert_eq!(destination, Some(PathBuf::from("./dist")));
            }
            _ => panic!("expected plan build command"),
        }
    }

    #[test]
    fn deploy_upload_syntax_remains_backward_compatible() {
        let cli = Cli::try_parse_from([
            "brume", "deploy", "./dist", "--url", "example", "--spa", "--pin",
        ])
        .expect("existing deploy upload syntax should keep parsing");

        match cli.command {
            Command::Deploy {
                command,
                directory,
                url,
                spa,
                pin,
                ..
            } => {
                assert!(command.is_none());
                assert_eq!(directory, PathBuf::from("./dist"));
                assert_eq!(url.as_deref(), Some("example"));
                assert!(spa);
                assert!(pin);
            }
            _ => panic!("expected deploy command"),
        }
    }

    #[test]
    fn deploy_list_is_parsed_as_a_subcommand() {
        let cli = Cli::try_parse_from(["brume", "deploy", "list", "--output", "json"])
            .expect("deploy list should parse");

        assert_eq!(cli.output, OutputFormat::Json);
        assert!(matches!(
            cli.command,
            Command::Deploy {
                command: Some(DeployCommand::List),
                ..
            }
        ));
    }

    #[test]
    fn deploy_delete_accepts_selector_and_confirmation() {
        let cli = Cli::try_parse_from(["brume", "deploy", "delete", "example", "--yes"])
            .expect("deploy delete should parse");

        assert!(matches!(
            cli.command,
            Command::Deploy {
                command: Some(DeployCommand::Delete {
                    deployment,
                    yes: true,
                }),
                ..
            } if deployment == "example"
        ));
    }

    #[test]
    fn archive_sizes_are_human_readable() {
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1_536), "1.5 KiB");
        assert_eq!(human_bytes(2 * 1024 * 1024), "2.0 MiB");
    }
}
