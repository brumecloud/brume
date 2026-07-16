use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEPLOYMENT_MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSummary {
    pub id: Uuid,
    pub owner_handle: String,
    pub slug: String,
    pub url: String,
    pub spa: bool,
    pub published_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploySiteResponse {
    pub deployment: DeploymentSummary,
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
