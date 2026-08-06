use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuthMode, ReviewRoundSummary};

pub const DEPLOYMENT_MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSummary {
    pub id: Uuid,
    pub owner_handle: String,
    pub slug: String,
    pub url: String,
    pub spa: bool,
    pub auth: AuthMode,
    pub overlay_enabled: bool,
    pub published_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploySiteResponse {
    pub deployment: DeploymentSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_round: Option<ReviewRoundSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDeploymentsResponse {
    pub deployments: Vec<DeploymentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeploymentDeletionChallengeResponse {
    pub challenge: String,
    pub expires_in_seconds: u64,
    pub deployment: DeploymentSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentManifest {
    pub spa: bool,
    pub files: Vec<DeploymentFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentFile {
    pub path: String,
    pub size: u64,
    pub content_type: String,
}
