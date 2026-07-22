use std::time::Duration;

use brume_core::{
    ApiErrorBody, BeginCliLoginResponse, ConfirmDeletionRequest, CreateDeletionChallengeResponse,
    DeployPlanResponse, DeploySiteResponse, ListPlansResponse, PlanDetails, PlanPatch,
    PollCliLoginResponse, RefreshTokenRequest, TokenPair, Visibility,
};
use reqwest::{Response, StatusCode};
use thiserror::Error;
use url::Url;

#[derive(Clone)]
pub struct BrumeClient {
    base_url: Url,
    http: reqwest::Client,
    token: Option<String>,
}

impl BrumeClient {
    pub fn new(base_url: &str, token: Option<String>) -> Result<Self, ClientError> {
        let base_url = Url::parse(base_url)?.join("/")?;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .user_agent(concat!("brume-cli/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            base_url,
            http,
            token,
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let request = self.http.request(
            method,
            self.base_url
                .join(path.trim_start_matches('/'))
                .expect("valid API path"),
        );
        match &self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }

    pub async fn begin_cli_login(&self) -> Result<BeginCliLoginResponse, ClientError> {
        decode(
            self.request(reqwest::Method::POST, "api/v1/auth/cli/sessions")
                .send()
                .await?,
        )
        .await
    }

    pub async fn poll_cli_login(
        &self,
        session_id: &str,
        poll_secret: &str,
    ) -> Result<PollCliLoginResponse, ClientError> {
        let path = format!(
            "api/v1/auth/cli/sessions/{}/poll",
            urlencoding::encode(session_id)
        );
        decode(
            self.request(reqwest::Method::POST, &path)
                .header("x-brume-poll-secret", poll_secret)
                .send()
                .await?,
        )
        .await
    }

    pub async fn refresh_token(&self, refresh_token: String) -> Result<TokenPair, ClientError> {
        decode(
            self.request(reqwest::Method::POST, "api/v1/auth/tokens/refresh")
                .json(&RefreshTokenRequest { refresh_token })
                .send()
                .await?,
        )
        .await
    }

    pub async fn deploy(
        &self,
        slug: &str,
        visibility: Visibility,
        pinned: bool,
        archive: Vec<u8>,
    ) -> Result<DeployPlanResponse, ClientError> {
        let path = format!(
            "api/v1/plans/{}/deploy?visibility={visibility}&pinned={pinned}",
            urlencoding::encode(slug)
        );
        decode(
            self.request(reqwest::Method::POST, &path)
                .header(reqwest::header::CONTENT_TYPE, "application/zstd")
                .body(archive)
                .send()
                .await?,
        )
        .await
    }

    pub async fn deploy_site(
        &self,
        slug: Option<&str>,
        spa: bool,
        pinned: bool,
        archive: Vec<u8>,
    ) -> Result<DeploySiteResponse, ClientError> {
        let mut path = format!("api/v1/deployments?spa={spa}&pinned={pinned}");
        if let Some(slug) = slug {
            path.push_str("&slug=");
            path.push_str(&urlencoding::encode(slug));
        }
        decode(
            self.request(reqwest::Method::POST, &path)
                .header(reqwest::header::CONTENT_TYPE, "application/zstd")
                .body(archive)
                .send()
                .await?,
        )
        .await
    }

    pub async fn list_plans(&self) -> Result<ListPlansResponse, ClientError> {
        decode(
            self.request(reqwest::Method::GET, "api/v1/plans")
                .send()
                .await?,
        )
        .await
    }

    pub async fn get_plan(&self, selector: &str) -> Result<PlanDetails, ClientError> {
        let path = format!("api/v1/plans/{}", urlencoding::encode(selector));
        decode(self.request(reqwest::Method::GET, &path).send().await?).await
    }

    pub async fn patch_plan(
        &self,
        selector: &str,
        patch: &PlanPatch,
    ) -> Result<PlanDetails, ClientError> {
        let path = format!("api/v1/plans/{}", urlencoding::encode(selector));
        decode(
            self.request(reqwest::Method::PATCH, &path)
                .json(patch)
                .send()
                .await?,
        )
        .await
    }

    pub async fn create_deletion_challenge(
        &self,
        selector: &str,
    ) -> Result<CreateDeletionChallengeResponse, ClientError> {
        let path = format!(
            "api/v1/plans/{}/deletion-challenges",
            urlencoding::encode(selector)
        );
        decode(self.request(reqwest::Method::POST, &path).send().await?).await
    }

    pub async fn confirm_deletion(
        &self,
        selector: &str,
        challenge: String,
    ) -> Result<(), ClientError> {
        let path = format!("api/v1/plans/{}", urlencoding::encode(selector));
        let response = self
            .request(reqwest::Method::DELETE, &path)
            .json(&ConfirmDeletionRequest { challenge })
            .send()
            .await?;
        if response.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(decode_error(response).await)
        }
    }
}

async fn decode<T: serde::de::DeserializeOwned>(response: Response) -> Result<T, ClientError> {
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(decode_error(response).await)
    }
}

async fn decode_error(response: Response) -> ClientError {
    let status = response.status();
    match response.json::<ApiErrorBody>().await {
        Ok(body) => ClientError::Api {
            status,
            code: body.code,
            message: body.message,
        },
        Err(error) => ClientError::UnexpectedResponse {
            status,
            source: error,
        },
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("invalid Brume URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Brume request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Brume API returned {status} ({code}): {message}")]
    Api {
        status: StatusCode,
        code: String,
        message: String,
    },
    #[error("Brume API returned {status} with an invalid body: {source}")]
    UnexpectedResponse {
        status: StatusCode,
        source: reqwest::Error,
    },
}
