use std::time::Duration;

use brume_core::{
    ApiErrorBody, AuthMode, BeginCliLoginResponse, ConfirmDeletionRequest,
    CreateDeletionChallengeResponse, CreateDeploymentDeletionChallengeResponse, DeployPlanResponse,
    DeploySiteResponse, DeploymentPatch, DeploymentSummary, ListDeploymentsResponse,
    ListPlansResponse, PlanDetails, PlanPatch, PollCliLoginResponse, RefreshTokenRequest,
    ReviewCommentsResponse, ReviewRoundsResponse, ReviewStatusResponse, TokenPair,
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

pub struct DeploySiteOptions<'a> {
    pub slug: Option<&'a str>,
    pub spa: bool,
    pub auth: AuthMode,
    pub password: Option<&'a str>,
    pub overlay_enabled: bool,
    pub pinned: bool,
    pub review: bool,
    pub archive: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewTarget {
    Plan,
    Deployment,
}

impl ReviewTarget {
    fn base_path(self, selector: &str) -> String {
        let collection = match self {
            Self::Plan => "plans",
            Self::Deployment => "deployments",
        };
        format!(
            "api/v1/{collection}/{}/review",
            urlencoding::encode(selector)
        )
    }
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

    #[allow(clippy::too_many_arguments)]
    pub async fn deploy(
        &self,
        slug: &str,
        auth: AuthMode,
        password: Option<&str>,
        overlay_enabled: bool,
        pinned: bool,
        review: bool,
        archive: Vec<u8>,
    ) -> Result<DeployPlanResponse, ClientError> {
        let path = format!(
            "api/v1/plans/{}/deploy?auth={auth}&overlay={overlay_enabled}&pinned={pinned}&review={review}",
            urlencoding::encode(slug),
        );
        let mut request = self
            .request(reqwest::Method::POST, &path)
            .header(reqwest::header::CONTENT_TYPE, "application/zstd");
        if let Some(password) = password {
            request = request.header("x-brume-password", password);
        }
        decode(request.body(archive).send().await?).await
    }

    pub async fn deploy_site(
        &self,
        options: DeploySiteOptions<'_>,
    ) -> Result<DeploySiteResponse, ClientError> {
        let DeploySiteOptions {
            slug,
            spa,
            auth,
            password,
            overlay_enabled,
            pinned,
            review,
            archive,
        } = options;
        let mut path = format!(
            "api/v1/deployments?spa={spa}&auth={auth}&overlay={overlay_enabled}&pinned={pinned}&review={review}"
        );
        if let Some(slug) = slug {
            path.push_str("&slug=");
            path.push_str(&urlencoding::encode(slug));
        }
        let mut request = self
            .request(reqwest::Method::POST, &path)
            .header(reqwest::header::CONTENT_TYPE, "application/zstd");
        if let Some(password) = password {
            request = request.header("x-brume-password", password);
        }
        decode(request.body(archive).send().await?).await
    }

    pub async fn list_deployments(&self) -> Result<ListDeploymentsResponse, ClientError> {
        decode(
            self.request(reqwest::Method::GET, "api/v1/deployments")
                .send()
                .await?,
        )
        .await
    }

    pub async fn get_deployment(&self, selector: &str) -> Result<DeploymentSummary, ClientError> {
        let path = format!("api/v1/deployments/{}", urlencoding::encode(selector));
        decode(self.request(reqwest::Method::GET, &path).send().await?).await
    }

    pub async fn patch_deployment(
        &self,
        selector: &str,
        patch: &DeploymentPatch,
    ) -> Result<DeploymentSummary, ClientError> {
        let path = format!("api/v1/deployments/{}", urlencoding::encode(selector));
        decode(
            self.request(reqwest::Method::PATCH, &path)
                .json(patch)
                .send()
                .await?,
        )
        .await
    }

    pub async fn create_deployment_deletion_challenge(
        &self,
        selector: &str,
    ) -> Result<CreateDeploymentDeletionChallengeResponse, ClientError> {
        let path = format!(
            "api/v1/deployments/{}/deletion-challenges",
            urlencoding::encode(selector)
        );
        decode(self.request(reqwest::Method::POST, &path).send().await?).await
    }

    pub async fn confirm_deployment_deletion(
        &self,
        selector: &str,
        challenge: String,
    ) -> Result<(), ClientError> {
        let path = format!("api/v1/deployments/{}", urlencoding::encode(selector));
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

    pub async fn review_status(
        &self,
        target: ReviewTarget,
        selector: &str,
    ) -> Result<ReviewStatusResponse, ClientError> {
        let path = target.base_path(selector);
        decode(self.request(reqwest::Method::GET, &path).send().await?).await
    }

    pub async fn review_comments(
        &self,
        target: ReviewTarget,
        selector: &str,
        round: Option<i32>,
    ) -> Result<ReviewCommentsResponse, ClientError> {
        let mut path = format!("{}/comments", target.base_path(selector));
        if let Some(round) = round {
            path.push_str(&format!("?round={round}"));
        }
        decode(self.request(reqwest::Method::GET, &path).send().await?).await
    }

    pub async fn review_rounds(
        &self,
        target: ReviewTarget,
        selector: &str,
    ) -> Result<ReviewRoundsResponse, ClientError> {
        let path = format!("{}/rounds", target.base_path(selector));
        decode(self.request(reqwest::Method::GET, &path).send().await?).await
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
