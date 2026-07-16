use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::{Context, Result, bail};
use brume_core::DEPLOYMENT_MAX_FILE_BYTES;
use tar::Builder;
use walkdir::WalkDir;

const MAX_DEPLOYMENT_BYTES: u64 = 100 * 1024 * 1024;
const MAX_DEPLOYMENT_FILES: usize = 5_000;

pub fn create_bundle_archive(directory: &Path) -> Result<Vec<u8>> {
    let encoder = zstd::Encoder::new(Vec::new(), 8)?;
    let mut archive = Builder::new(encoder);
    archive.follow_symlinks(false);

    for entry in WalkDir::new(directory).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.file_type().is_symlink() {
            bail!(
                "bundle contains a symbolic link: {}",
                entry.path().display()
            );
        }
        let relative = entry.path().strip_prefix(directory)?;
        let mut file = File::open(entry.path())
            .with_context(|| format!("opening {}", entry.path().display()))?;
        archive.append_file(relative, &mut file)?;
    }
    let encoder = archive.into_inner()?;
    Ok(encoder.finish()?)
}

pub fn create_deployment_archive(directory: &Path) -> Result<Vec<u8>> {
    let entrypoint = directory.join("index.html");
    let entrypoint_metadata = fs::symlink_metadata(&entrypoint)
        .with_context(|| format!("opening deployment entrypoint {}", entrypoint.display()))?;
    if !entrypoint_metadata.file_type().is_file() {
        bail!(
            "deployment entrypoint {} is not a regular file",
            entrypoint.display()
        );
    }

    let encoder = zstd::Encoder::new(Vec::new(), 8)?;
    let mut archive = Builder::new(encoder);
    archive.follow_symlinks(false);
    let mut total_size = 0_u64;
    let mut file_count = 0_usize;

    for entry in WalkDir::new(directory).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            bail!(
                "deployment contains a link or unsupported filesystem entry: {}",
                entry.path().display()
            );
        }
        file_count += 1;
        if file_count > MAX_DEPLOYMENT_FILES {
            bail!("deployment contains more than {MAX_DEPLOYMENT_FILES} files");
        }
        let size = entry.metadata()?.len();
        if size > DEPLOYMENT_MAX_FILE_BYTES {
            bail!(
                "deployment object {} exceeds 20 MiB",
                entry.path().display()
            );
        }
        total_size += size;
        if total_size > MAX_DEPLOYMENT_BYTES {
            bail!("deployment exceeds 100 MiB");
        }
        let relative = entry.path().strip_prefix(directory)?;
        let archive_path = relative
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("deployment path is not valid UTF-8"))?
            .replace('\\', "/");
        let mut file = File::open(entry.path())
            .with_context(|| format!("opening {}", entry.path().display()))?;
        archive.append_file(archive_path, &mut file)?;
    }
    let encoder = archive.into_inner()?;
    Ok(encoder.finish()?)
}
