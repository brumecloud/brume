mod archive;
mod config;
mod embedded;
mod output;
mod preview;
mod renderer;
mod tunnel;

mod build_metadata {
    include!(concat!(env!("OUT_DIR"), "/build_metadata.rs"));
}

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use brume_api_client::BrumeClient;
use brume_core::{PlanPatch, PollCliLoginResponse, Visibility};
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
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        spa: bool,
        #[arg(long)]
        pin: bool,
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
        visibility: Option<Visibility>,
        #[arg(long)]
        pin: bool,
    },
    List,
    Show {
        plan: String,
    },
    Open {
        plan: String,
    },
    Visibility {
        plan: String,
        visibility: Visibility,
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
            directory,
            url,
            spa,
            pin,
        } => deploy(&cli.base_url, &directory, url, spa, pin, cli.output).await,
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

async fn deploy(
    base_url: &str,
    directory: &Path,
    slug: Option<String>,
    spa: bool,
    pin: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let token = config::load_access_token(base_url).await?;
    let source = canonical_directory(directory)?;
    if let Some(slug) = &slug {
        config::validate_slug(slug)?;
    }
    let archive = archive::create_deployment_archive(&source)?;
    let deployed = BrumeClient::new(base_url, Some(token))?
        .deploy_site(slug.as_deref(), spa, pin, archive)
        .await?;
    if output_format.is_json() {
        output::json(&deployed)?;
    } else {
        println!("Deployed {}", deployed.deployment.url);
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
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let temporary = TempDir::new()?;
            let rendered = renderer::render(
                &source,
                temporary.path(),
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
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
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let destination =
                absolute_path(destination.unwrap_or_else(|| source.join(".brume").join("dist")))?;
            let rendered = renderer::render(
                &source,
                &destination,
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
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
            visibility,
            pin,
        } => {
            let token = config::load_access_token(base_url).await?;
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let slug = slug
                .or(project.plan.slug.clone())
                .map(Ok)
                .unwrap_or_else(|| config::default_slug(&source))?;
            config::validate_slug(&slug)?;
            let visibility = visibility
                .or(project.plan.visibility)
                .unwrap_or(Visibility::Private);
            let temporary = TempDir::new()?;
            renderer::render(
                &source,
                temporary.path(),
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
            let archive = archive::create_bundle_archive(temporary.path())?;
            let deployed = BrumeClient::new(base_url, Some(token))?
                .deploy(&slug, visibility, pin, archive)
                .await?;
            if output_format.is_json() {
                output::json(&deployed)?;
            } else {
                println!("Deployed {}", deployed.plan.summary.url);
                if let Some(url) = deployed.unlisted_url {
                    println!("Unlisted URL: {url}");
                }
            }
            Ok(())
        }
        PlanCommand::List => {
            let response = authenticated_client(base_url).await?.list_plans().await?;
            if output_format.is_json() {
                output::json(&response)?;
            } else {
                output::plans(&response)?;
            }
            Ok(())
        }
        PlanCommand::Show { plan } => {
            let details = authenticated_client(base_url)
                .await?
                .get_plan(&plan)
                .await?;
            if output_format.is_json() {
                output::json(&details)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&details)?);
            }
            Ok(())
        }
        PlanCommand::Open { plan } => {
            let details = authenticated_client(base_url)
                .await?
                .get_plan(&plan)
                .await?;
            open::that(&details.summary.url)?;
            if output_format.is_json() {
                output::json(&details)?;
            }
            Ok(())
        }
        PlanCommand::Visibility { plan, visibility } => {
            let details = authenticated_client(base_url)
                .await?
                .patch_plan(
                    &plan,
                    &PlanPatch {
                        visibility: Some(visibility),
                        pinned: None,
                    },
                )
                .await?;
            if output_format.is_json() {
                output::json(&details)?;
            } else {
                println!(
                    "{} is now {}",
                    details.summary.slug, details.summary.visibility
                );
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
    let details = authenticated_client(base_url)
        .await?
        .patch_plan(
            plan,
            &PlanPatch {
                visibility: None,
                pinned: Some(pinned),
            },
        )
        .await?;
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
    let client = authenticated_client(base_url).await?;
    let challenge = client.create_deletion_challenge(plan).await?;
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
    client.confirm_deletion(plan, challenge.challenge).await?;
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
}
