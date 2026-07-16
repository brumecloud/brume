use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Unlisted,
    Public,
}

impl fmt::Display for Visibility {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Private => "private",
            Self::Unlisted => "unlisted",
            Self::Public => "public",
        })
    }
}

impl FromStr for Visibility {
    type Err = ParseVisibilityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "private" => Ok(Self::Private),
            "unlisted" => Ok(Self::Unlisted),
            "public" => Ok(Self::Public),
            _ => Err(ParseVisibilityError(value.to_owned())),
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown visibility `{0}`; expected private, unlisted, or public")]
pub struct ParseVisibilityError(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub id: Uuid,
    pub owner_handle: String,
    pub slug: String,
    pub title: String,
    pub visibility: Visibility,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDetails {
    #[serde(flatten)]
    pub summary: PlanSummary,
    pub renderer_version: String,
    pub html_contract_version: u32,
    pub routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}
