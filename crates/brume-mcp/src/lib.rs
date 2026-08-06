use std::{future::Future, pin::Pin, process::Stdio, sync::Arc};

use anyhow::Result;
use brume_api_client::{BrumeClient, ReviewTarget};
use brume_core::{AuthMode, PlanPatch};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PlanSelector {
    /// A plan UUID or slug owned by the authenticated user.
    plan: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AuthRequest {
    /// A plan UUID or slug owned by the authenticated user.
    plan: String,
    /// One of token, password, or none.
    auth: String,
    /// Required when switching to password authentication.
    password: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RetentionRequest {
    /// A plan UUID or slug owned by the authenticated user.
    plan: String,
    /// True keeps the plan indefinitely. False restores the 15 day retention policy.
    pinned: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ConfirmDeletionRequest {
    /// A plan UUID or slug owned by the authenticated user.
    plan: String,
    /// The short-lived challenge returned by plan_delete_prepare.
    challenge: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeployRequest {
    /// Absolute or current-working-directory-relative path to the plan folder.
    directory: String,
    /// Stable lowercase URL slug.
    slug: Option<String>,
    /// One of token, password, or none. Defaults to the project configuration or token.
    auth: Option<String>,
    /// Required when auth is password. It is passed to the CLI through stdin.
    password: Option<String>,
    /// True prevents automatic deletion after 15 days without a read.
    #[serde(default)]
    pinned: bool,
    /// True starts a review round so reviewers can comment on the deployed plan.
    #[serde(default)]
    review: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReviewSelector {
    /// A plan or deployment UUID or slug owned by the authenticated user.
    slug: String,
    /// True targets a static deployment instead of a plan.
    #[serde(default)]
    site: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReviewCommentsRequest {
    /// A plan or deployment UUID or slug owned by the authenticated user.
    slug: String,
    /// True targets a static deployment instead of a plan.
    #[serde(default)]
    site: bool,
    /// A specific review round number. Defaults to the latest round.
    round: Option<i32>,
}

fn review_target(site: bool) -> ReviewTarget {
    if site {
        ReviewTarget::Deployment
    } else {
        ReviewTarget::Plan
    }
}

#[derive(Clone)]
struct BrumeMcp {
    base_url: String,
    token_loader: TokenLoader,
    tool_router: ToolRouter<Self>,
}

pub type TokenFuture = Pin<Box<dyn Future<Output = Result<String, String>> + Send>>;
pub type TokenLoader = Arc<dyn Fn() -> TokenFuture + Send + Sync>;

#[tool_router]
impl BrumeMcp {
    fn new(base_url: String, token_loader: TokenLoader) -> Self {
        Self {
            base_url,
            token_loader,
            tool_router: Self::tool_router(),
        }
    }

    async fn client(&self) -> Result<BrumeClient, String> {
        let token = (self.token_loader)().await?;
        BrumeClient::new(&self.base_url, Some(token)).map_err(|error| error.to_string())
    }

    fn command(&self) -> Result<Command, String> {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut command = Command::new(executable);
        command
            .arg("--base-url")
            .arg(&self.base_url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        Ok(command)
    }

    #[tool(
        description = "List Brume plans with authentication, last read time, expiry, pin state, and URL"
    )]
    async fn plans_list(&self) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client.list_plans().await {
            Ok(plans) => json(&plans),
            Err(error) => tool_error(error),
        }
    }

    #[tool(description = "Get metadata and routes for one Brume plan")]
    async fn plan_get(&self, Parameters(request): Parameters<PlanSelector>) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client.get_plan(&request.plan).await {
            Ok(plan) => json(&plan),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Deploy a local Markdown or MDX plan directory using the embedded Brume renderer"
    )]
    async fn plan_deploy(&self, Parameters(request): Parameters<DeployRequest>) -> String {
        let mut command = match self.command() {
            Ok(command) => command,
            Err(error) => return tool_error(error),
        };
        command.arg("plan").arg("deploy").arg(&request.directory);
        if let Some(slug) = request.slug {
            command.arg("--slug").arg(slug);
        }
        if let Some(auth) = request.auth {
            command.arg("--auth").arg(auth);
        }
        if request.password.is_some() {
            command.arg("--password-stdin");
            command.stdin(Stdio::piped());
        }
        if request.pinned {
            command.arg("--pin");
        }
        if request.review {
            command.arg("--review");
        }
        match command_output(command, request.password).await {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            }
            Ok(output) => format!(
                "Brume deploy failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => tool_error(error),
        }
    }

    #[tool(description = "Change a Brume plan authentication mode to token, password, or none")]
    async fn plan_set_auth(&self, Parameters(request): Parameters<AuthRequest>) -> String {
        let auth = match request.auth.parse::<AuthMode>() {
            Ok(value) => value,
            Err(error) => return tool_error(error),
        };
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client
            .patch_plan(
                &request.plan,
                &PlanPatch {
                    auth: Some(auth),
                    password: request.password,
                    overlay_enabled: None,
                    pinned: None,
                },
            )
            .await
        {
            Ok(plan) => json(&plan),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Pin or unpin a plan. Pinned plans are excluded from automatic retention deletion"
    )]
    async fn plan_set_pinned(&self, Parameters(request): Parameters<RetentionRequest>) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client
            .patch_plan(
                &request.plan,
                &PlanPatch {
                    auth: None,
                    password: None,
                    overlay_enabled: None,
                    pinned: Some(request.pinned),
                },
            )
            .await
        {
            Ok(plan) => json(&plan),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Get the review status of a plan or static deployment. Poll this until status is `finished`, then call review_comments to fetch the reviewer feedback"
    )]
    async fn review_status(&self, Parameters(request): Parameters<ReviewSelector>) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client
            .review_status(review_target(request.site), &request.slug)
            .await
        {
            Ok(status) => json(&status),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Fetch the threaded reviewer comments of a plan or static deployment review round, including the highlighted text or element each thread is anchored to"
    )]
    async fn review_comments(
        &self,
        Parameters(request): Parameters<ReviewCommentsRequest>,
    ) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client
            .review_comments(review_target(request.site), &request.slug, request.round)
            .await
        {
            Ok(comments) => json(&comments),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Prepare permanent plan deletion and return a short-lived confirmation challenge"
    )]
    async fn plan_delete_prepare(&self, Parameters(request): Parameters<PlanSelector>) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client.create_deletion_challenge(&request.plan).await {
            Ok(challenge) => json(&challenge),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Permanently delete a plan using the challenge returned by plan_delete_prepare"
    )]
    async fn plan_delete_confirm(
        &self,
        Parameters(request): Parameters<ConfirmDeletionRequest>,
    ) -> String {
        let client = match self.client().await {
            Ok(client) => client,
            Err(error) => return tool_error(error),
        };
        match client
            .confirm_deletion(&request.plan, request.challenge)
            .await
        {
            Ok(()) => format!("Deleted {}", request.plan),
            Err(error) => tool_error(error),
        }
    }
}

async fn command_output(
    mut command: Command,
    password: Option<String>,
) -> std::io::Result<std::process::Output> {
    if let Some(password) = password {
        let mut child = command.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(password.as_bytes()).await?;
        }
        child.wait_with_output().await
    } else {
        command.output().await
    }
}

#[tool_handler]
impl ServerHandler for BrumeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Publish and manage Brume Markdown or MDX plans. Permanent deletion always requires prepare then confirm."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

pub async fn serve(base_url: &str, token_loader: TokenLoader) -> Result<()> {
    let service = BrumeMcp::new(base_url.to_owned(), token_loader)
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}

fn json(value: &impl serde::Serialize) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|error| format!("Serialization failed: {error}"))
}

fn tool_error(error: impl std::fmt::Display) -> String {
    format!("Brume operation failed: {error}")
}
