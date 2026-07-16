use std::process::Stdio;

use anyhow::Result;
use brume_api_client::BrumeClient;
use brume_core::{PlanPatch, Visibility};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PlanSelector {
    /// A plan UUID or slug owned by the authenticated user.
    plan: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct VisibilityRequest {
    /// A plan UUID or slug owned by the authenticated user.
    plan: String,
    /// One of private, unlisted, or public.
    visibility: String,
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
    /// One of private, unlisted, or public. Defaults to the project configuration.
    visibility: Option<String>,
    /// True prevents automatic deletion after 15 days without a read.
    #[serde(default)]
    pinned: bool,
}

#[derive(Clone)]
struct BrumeMcp {
    base_url: String,
    token: String,
    client: BrumeClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BrumeMcp {
    fn new(base_url: String, token: String) -> Result<Self> {
        Ok(Self {
            client: BrumeClient::new(&base_url, Some(token.clone()))?,
            base_url,
            token,
            tool_router: Self::tool_router(),
        })
    }

    #[tool(
        description = "List Brume plans with visibility, last read time, expiry, pin state, and URL"
    )]
    async fn plans_list(&self) -> String {
        match self.client.list_plans().await {
            Ok(plans) => json(&plans),
            Err(error) => tool_error(error),
        }
    }

    #[tool(description = "Get metadata and routes for one Brume plan")]
    async fn plan_get(&self, Parameters(request): Parameters<PlanSelector>) -> String {
        match self.client.get_plan(&request.plan).await {
            Ok(plan) => json(&plan),
            Err(error) => tool_error(error),
        }
    }

    #[tool(
        description = "Deploy a local Markdown or MDX plan directory using the embedded Brume renderer"
    )]
    async fn plan_deploy(&self, Parameters(request): Parameters<DeployRequest>) -> String {
        let executable = match std::env::current_exe() {
            Ok(value) => value,
            Err(error) => return tool_error(error),
        };
        let mut command = Command::new(executable);
        command
            .arg("--base-url")
            .arg(&self.base_url)
            .arg("plan")
            .arg("deploy")
            .arg(&request.directory)
            .env("BRUME_TOKEN", &self.token)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(slug) = request.slug {
            command.arg("--slug").arg(slug);
        }
        if let Some(visibility) = request.visibility {
            command.arg("--visibility").arg(visibility);
        }
        if request.pinned {
            command.arg("--pin");
        }
        match command.output().await {
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

    #[tool(description = "Change a Brume plan visibility to private, unlisted, or public")]
    async fn plan_set_visibility(
        &self,
        Parameters(request): Parameters<VisibilityRequest>,
    ) -> String {
        let visibility = match request.visibility.parse::<Visibility>() {
            Ok(value) => value,
            Err(error) => return tool_error(error),
        };
        match self
            .client
            .patch_plan(
                &request.plan,
                &PlanPatch {
                    visibility: Some(visibility),
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
        match self
            .client
            .patch_plan(
                &request.plan,
                &PlanPatch {
                    visibility: None,
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
        description = "Prepare permanent plan deletion and return a short-lived confirmation challenge"
    )]
    async fn plan_delete_prepare(&self, Parameters(request): Parameters<PlanSelector>) -> String {
        match self.client.create_deletion_challenge(&request.plan).await {
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
        match self
            .client
            .confirm_deletion(&request.plan, request.challenge)
            .await
        {
            Ok(()) => format!("Deleted {}", request.plan),
            Err(error) => tool_error(error),
        }
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

pub async fn serve(base_url: &str, token: String) -> Result<()> {
    let service = BrumeMcp::new(base_url.to_owned(), token)?
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
