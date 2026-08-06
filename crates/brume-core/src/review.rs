use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Open,
    Finished,
    Superseded,
}

impl fmt::Display for ReviewStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Open => "open",
            Self::Finished => "finished",
            Self::Superseded => "superseded",
        })
    }
}

impl FromStr for ReviewStatus {
    type Err = ParseReviewStatusError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "open" => Ok(Self::Open),
            "finished" => Ok(Self::Finished),
            "superseded" => Ok(Self::Superseded),
            _ => Err(ParseReviewStatusError(value.to_owned())),
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown review status `{0}`; expected open, finished, or superseded")]
pub struct ParseReviewStatusError(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRoundSummary {
    pub id: Uuid,
    pub number: i32,
    pub status: ReviewStatus,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub thread_count: i64,
    pub comment_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStatusResponse {
    pub round: Option<ReviewRoundSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRoundsResponse {
    pub rounds: Vec<ReviewRoundSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: Uuid,
    pub author_public_id: Option<String>,
    pub author_display_name: Option<String>,
    pub author_is_owner: bool,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewThread {
    pub id: Uuid,
    pub page_path: String,
    pub anchor: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub comments: Vec<ReviewComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCommentsResponse {
    pub round: ReviewRoundSummary,
    pub threads: Vec<ReviewThread>,
}
