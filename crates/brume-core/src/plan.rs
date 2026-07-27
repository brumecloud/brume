use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    #[default]
    Token,
    Password,
    None,
}

impl fmt::Display for AuthMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Token => "token",
            Self::Password => "password",
            Self::None => "none",
        })
    }
}

impl FromStr for AuthMode {
    type Err = ParseAuthModeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "token" => Ok(Self::Token),
            "password" => Ok(Self::Password),
            "none" => Ok(Self::None),
            _ => Err(ParseAuthModeError(value.to_owned())),
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown authentication mode `{0}`; expected token, password, or none")]
pub struct ParseAuthModeError(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub id: Uuid,
    pub owner_handle: String,
    pub slug: String,
    pub title: String,
    pub auth: AuthMode,
    pub overlay_enabled: bool,
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
    pub auth: Option<AuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}
