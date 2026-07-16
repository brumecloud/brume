use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{PlanDetails, PlanSummary, Visibility};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginCliLoginResponse {
    pub session_id: Uuid,
    pub browser_url: String,
    pub poll_secret: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PollCliLoginResponse {
    Pending,
    Authorized { token: String, user_handle: String },
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployPlanResponse {
    pub plan: PlanDetails,
    pub unlisted_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPlansResponse {
    pub plans: Vec<PlanSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeletionChallengeResponse {
    pub challenge: String,
    pub expires_in_seconds: u64,
    pub plan: PlanSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmDeletionRequest {
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployQuery {
    pub visibility: Visibility,
    pub pinned: bool,
}
