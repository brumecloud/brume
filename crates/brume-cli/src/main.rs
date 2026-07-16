mod archive;
mod config;
mod embedded;
mod preview;
mod renderer;

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use brume_api_client::BrumeClient;
use brume_core::{PlanPatch, PollCliLoginResponse, Visibility};
use clap::{Parser, Subcommand};
use tempfile::TempDir;

#[derive(Parser)]
#[command(
    name = "brume",
    version,
    about = "Publish agent plans as durable documentation"
)]
struct Cli {
    #[arg(long, env = "BRUME_BASE_URL", default_value = "https://brume.dev")]
    base_url: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Login,
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
        #[arg(long)]
        output: Option<PathBuf>,
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
        Command::Login => login(&cli.base_url).await,
        Command::Mcp { command } => mcp(&cli.base_url, command).await,
        Command::Plan { command } => plan(&cli.base_url, command).await,
    }
}

async fn login(base_url: &str) -> Result<()> {
    let client = BrumeClient::new(base_url, None)?;
    let session = client.begin_cli_login().await?;
    println!("Opening {}", session.browser_url);
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
            PollCliLoginResponse::Authorized { token, user_handle } => {
                config::save_token(base_url, &token)?;
                println!("Logged in as @{user_handle}");
                return Ok(());
            }
            PollCliLoginResponse::Expired => {
                bail!("login session expired; run `brume login` again")
            }
        }
    }
}

async fn mcp(base_url: &str, command: McpCommand) -> Result<()> {
    match command {
        McpCommand::Serve => {
            let token = config::load_token(base_url)?;
            brume_mcp::serve(base_url, token).await
        }
        McpCommand::Config => {
            println!(
                "[mcp_servers.brume]\ncommand = \"brume\"\nargs = [\"--base-url\", \"{base_url}\", \"mcp\", \"serve\"]"
            );
            Ok(())
        }
    }
}

async fn plan(base_url: &str, command: PlanCommand) -> Result<()> {
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
            println!(
                "Rendered {} pages and {} assets",
                rendered.page_count, rendered.asset_count
            );
            preview::serve(
                temporary.path().to_path_buf(),
                rendered.manifest,
                port,
                !no_open,
            )
            .await
        }
        PlanCommand::Build { directory, output } => {
            let source = canonical_directory(&directory)?;
            let project = config::load_project(&source)?;
            let destination =
                absolute_path(output.unwrap_or_else(|| source.join(".brume").join("dist")))?;
            let rendered = renderer::render(
                &source,
                &destination,
                project.plan.entry.as_deref(),
                project.plan.title.as_deref(),
            )
            .await?;
            println!(
                "Built {} pages and {} assets in {}",
                rendered.page_count,
                rendered.asset_count,
                destination.display()
            );
            Ok(())
        }
        PlanCommand::Deploy {
            directory,
            slug,
            visibility,
            pin,
        } => {
            let token = config::load_token(base_url)?;
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
            println!("Deployed {}", deployed.plan.summary.url);
            if let Some(url) = deployed.unlisted_url {
                println!("Unlisted URL: {url}");
            }
            Ok(())
        }
        PlanCommand::List => {
            let plans = authenticated_client(base_url)?.list_plans().await?.plans;
            println!(
                "{:<28} {:<10} {:<22} {:<22} URL",
                "SLUG", "VISIBILITY", "LAST READ", "EXPIRES"
            );
            for plan in plans {
                let last_read = plan
                    .last_read_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_else(|| "never".to_owned());
                let expires = plan
                    .expires_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_else(|| "pinned".to_owned());
                println!(
                    "{:<28} {:<10} {:<22} {:<22} {}",
                    plan.slug, plan.visibility, last_read, expires, plan.url
                );
            }
            Ok(())
        }
        PlanCommand::Show { plan } => {
            let details = authenticated_client(base_url)?.get_plan(&plan).await?;
            println!("{}", serde_json::to_string_pretty(&details)?);
            Ok(())
        }
        PlanCommand::Open { plan } => {
            let details = authenticated_client(base_url)?.get_plan(&plan).await?;
            open::that(&details.summary.url)?;
            Ok(())
        }
        PlanCommand::Visibility { plan, visibility } => {
            let details = authenticated_client(base_url)?
                .patch_plan(
                    &plan,
                    &PlanPatch {
                        visibility: Some(visibility),
                        pinned: None,
                    },
                )
                .await?;
            println!(
                "{} is now {}",
                details.summary.slug, details.summary.visibility
            );
            Ok(())
        }
        PlanCommand::Pin { plan } => patch_pin(base_url, &plan, true).await,
        PlanCommand::Unpin { plan } => patch_pin(base_url, &plan, false).await,
        PlanCommand::Delete { plan, yes } => delete_plan(base_url, &plan, yes).await,
    }
}

async fn patch_pin(base_url: &str, plan: &str, pinned: bool) -> Result<()> {
    let details = authenticated_client(base_url)?
        .patch_plan(
            plan,
            &PlanPatch {
                visibility: None,
                pinned: Some(pinned),
            },
        )
        .await?;
    println!(
        "{} is {}",
        details.summary.slug,
        if pinned {
            "pinned"
        } else {
            "subject to retention"
        }
    );
    Ok(())
}

async fn delete_plan(base_url: &str, plan: &str, yes: bool) -> Result<()> {
    let client = authenticated_client(base_url)?;
    let challenge = client.create_deletion_challenge(plan).await?;
    if !yes {
        print!(
            "Delete `{}` and all of its files permanently? [y/N] ",
            challenge.plan.slug
        );
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
            println!("Deletion cancelled");
            return Ok(());
        }
    }
    client.confirm_deletion(plan, challenge.challenge).await?;
    println!("Deleted {}", challenge.plan.slug);
    Ok(())
}

fn authenticated_client(base_url: &str) -> Result<BrumeClient> {
    BrumeClient::new(base_url, Some(config::load_token(base_url)?)).map_err(Into::into)
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
