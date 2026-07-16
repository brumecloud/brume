use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const BUNDLE_FORMAT_VERSION: u32 = 1;
pub const HTML_CONTRACT_VERSION: u32 = 1;
pub const BASE_PATH_PLACEHOLDER: &str = "__BRUME_BASE_PATH__";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub format_version: u32,
    pub renderer_version: String,
    pub html_contract_version: u32,
    pub title: String,
    pub entry: String,
    pub pages: Vec<PageManifest>,
    pub assets: Vec<AssetManifest>,
    pub sources: Vec<SourceManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageManifest {
    pub route: String,
    pub object_path: String,
    pub source_path: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub path: String,
    pub sha256: String,
    pub size: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

impl BundleManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.format_version != BUNDLE_FORMAT_VERSION {
            return Err(ManifestError::UnsupportedBundleVersion(self.format_version));
        }
        if self.html_contract_version != HTML_CONTRACT_VERSION {
            return Err(ManifestError::UnsupportedHtmlContract(
                self.html_contract_version,
            ));
        }
        if self.title.trim().is_empty() {
            return Err(ManifestError::EmptyTitle);
        }
        if self.pages.is_empty() {
            return Err(ManifestError::NoPages);
        }
        if !self.pages.iter().any(|page| page.route == "/") {
            return Err(ManifestError::MissingRootRoute);
        }

        for page in &self.pages {
            validate_route(&page.route)?;
            validate_relative_path(&page.object_path)?;
            validate_relative_path(&page.source_path)?;
        }
        for asset in &self.assets {
            validate_relative_path(&asset.path)?;
        }
        for source in &self.sources {
            validate_relative_path(&source.path)?;
        }
        Ok(())
    }
}

fn validate_route(route: &str) -> Result<(), ManifestError> {
    if !route.starts_with('/') || route.contains("..") || route.contains(['?', '#']) {
        return Err(ManifestError::UnsafeRoute(route.to_owned()));
    }
    Ok(())
}

pub fn validate_relative_path(value: &str) -> Result<(), ManifestError> {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        return Err(ManifestError::UnsafePath(value.to_owned()));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(ManifestError::UnsafePath(value.to_owned()));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("unsupported bundle format version {0}")]
    UnsupportedBundleVersion(u32),
    #[error("unsupported HTML contract version {0}")]
    UnsupportedHtmlContract(u32),
    #[error("bundle title cannot be empty")]
    EmptyTitle,
    #[error("bundle must contain at least one page")]
    NoPages,
    #[error("bundle must contain a root route")]
    MissingRootRoute,
    #[error("unsafe route `{0}`")]
    UnsafeRoute(String),
    #[error("unsafe relative path `{0}`")]
    UnsafePath(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_directory_paths() {
        assert!(matches!(
            validate_relative_path("../secret"),
            Err(ManifestError::UnsafePath(_))
        ));
    }

    #[test]
    fn accepts_nested_relative_paths() {
        assert!(validate_relative_path("routes/architecture/index.html").is_ok());
    }
}
