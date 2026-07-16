use std::{io::ErrorKind, path::PathBuf, sync::Arc};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
    primitives::ByteStream,
    types::{Delete, ObjectIdentifier},
};
use brume_core::validate_relative_path;
use bytes::Bytes;
use uuid::Uuid;

use crate::config::StorageConfig;

pub struct StoredObject {
    pub bytes: Bytes,
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put(&self, key: &str, bytes: Bytes, content_type: &str) -> Result<()>;
    async fn get(&self, key: &str) -> Result<StoredObject>;
    async fn delete_prefix(&self, prefix: &str) -> Result<()>;
}

pub async fn create(config: &StorageConfig) -> Result<Arc<dyn ObjectStore>> {
    match config {
        StorageConfig::Filesystem { path } => {
            tokio::fs::create_dir_all(path).await?;
            Ok(Arc::new(FilesystemStore { root: path.clone() }))
        }
        StorageConfig::S3 {
            endpoint,
            region,
            bucket,
            access_key_id,
            secret_access_key,
            force_path_style,
        } => {
            let credentials = Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "brume-railway-bucket",
            );
            let configuration = aws_sdk_s3::Config::builder()
                .behavior_version(BehaviorVersion::latest())
                .credentials_provider(credentials)
                .endpoint_url(endpoint)
                .force_path_style(*force_path_style)
                .region(Region::new(region.clone()))
                .build();
            Ok(Arc::new(S3Store {
                bucket: bucket.clone(),
                client: Client::from_conf(configuration),
            }))
        }
    }
}

struct FilesystemStore {
    root: PathBuf,
}

impl FilesystemStore {
    fn path(&self, key: &str) -> Result<PathBuf> {
        validate_relative_path(key)?;
        Ok(self.root.join(key))
    }
}

#[async_trait]
impl ObjectStore for FilesystemStore {
    async fn put(&self, key: &str, bytes: Bytes, _content_type: &str) -> Result<()> {
        let destination = self.path(key)?;
        let parent = destination
            .parent()
            .ok_or_else(|| anyhow!("object key has no parent"))?;
        tokio::fs::create_dir_all(parent).await?;
        let temporary = parent.join(format!(".upload-{}", Uuid::new_v4()));
        tokio::fs::write(&temporary, bytes).await?;
        tokio::fs::rename(&temporary, &destination).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<StoredObject> {
        let path = self.path(key)?;
        Ok(StoredObject {
            bytes: Bytes::from(
                tokio::fs::read(&path)
                    .await
                    .with_context(|| format!("reading object {}", path.display()))?,
            ),
        })
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let path = self.path(prefix.trim_end_matches('/'))?;
        match tokio::fs::remove_dir_all(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

struct S3Store {
    bucket: String,
    client: Client,
}

#[async_trait]
impl ObjectStore for S3Store {
    async fn put(&self, key: &str, bytes: Bytes, content_type: &str) -> Result<()> {
        validate_relative_path(key)?;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<StoredObject> {
        validate_relative_path(key)?;
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(StoredObject {
            bytes: response.body.collect().await?.into_bytes(),
        })
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        validate_relative_path(prefix.trim_end_matches('/'))?;
        let mut continuation_token = None;
        loop {
            let page = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .set_continuation_token(continuation_token)
                .send()
                .await?;
            let objects = page
                .contents()
                .iter()
                .filter_map(|object| object.key())
                .map(|key| ObjectIdentifier::builder().key(key).build())
                .collect::<Result<Vec<_>, _>>()?;
            if !objects.is_empty() {
                self.client
                    .delete_objects()
                    .bucket(&self.bucket)
                    .delete(Delete::builder().set_objects(Some(objects)).build()?)
                    .send()
                    .await?;
            }
            if !page.is_truncated().unwrap_or(false) {
                break;
            }
            continuation_token = page.next_continuation_token().map(ToOwned::to_owned);
        }
        Ok(())
    }
}
