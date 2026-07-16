use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{config::Config, storage};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub database: PgPool,
    pub http: Client,
    pub storage: Arc<dyn storage::ObjectStore>,
}

impl AppState {
    pub async fn initialize(config: Config) -> Result<Self> {
        let database = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await?;
        sqlx::migrate!("../../migrations").run(&database).await?;
        let storage = storage::create(&config.storage).await?;
        let http = Client::builder()
            .user_agent(concat!("brume-server/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            config: Arc::new(config),
            database,
            http,
            storage,
        })
    }
}
