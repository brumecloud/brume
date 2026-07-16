use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use brume_core::BundleManifest;
use directories::ProjectDirs;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{io::AsyncWriteExt, process::Command, time::timeout};

use crate::embedded::RENDERER_WORKER;

#[derive(Serialize)]
struct RenderRequest<'a> {
    source_dir: &'a Path,
    output_dir: &'a Path,
    #[serde(skip_serializing_if = "Option::is_none")]
    entry: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RenderResponse {
    Success {
        ok: bool,
        manifest_path: PathBuf,
        page_count: usize,
        asset_count: usize,
    },
    Failure {
        #[serde(rename = "ok")]
        _ok: bool,
        message: String,
        file: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
    },
}

pub struct RenderedPlan {
    pub manifest: BundleManifest,
    pub page_count: usize,
    pub asset_count: usize,
}

pub async fn render(
    source_dir: &Path,
    output_dir: &Path,
    entry: Option<&str>,
    title: Option<&str>,
) -> Result<RenderedPlan> {
    let worker = extract_worker()?;
    let request = serde_json::to_vec(&RenderRequest {
        source_dir,
        output_dir,
        entry,
        title,
    })?;
    let mut child = Command::new(&worker)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("starting embedded renderer {}", worker.display()))?;
    child
        .stdin
        .take()
        .expect("renderer stdin")
        .write_all(&request)
        .await?;

    let output = timeout(Duration::from_secs(120), child.wait_with_output())
        .await
        .context("renderer exceeded the two minute time limit")??;
    let response: RenderResponse = serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "renderer returned invalid JSON: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })?;
    match response {
        RenderResponse::Success {
            ok,
            manifest_path,
            page_count,
            asset_count,
        } if ok && output.status.success() => {
            let manifest: BundleManifest = serde_json::from_slice(
                &fs::read(&manifest_path)
                    .with_context(|| format!("reading {}", manifest_path.display()))?,
            )?;
            manifest.validate()?;
            Ok(RenderedPlan {
                manifest,
                page_count,
                asset_count,
            })
        }
        RenderResponse::Failure {
            message,
            file,
            line,
            column,
            ..
        } => {
            let location = match (file, line, column) {
                (Some(file), Some(line), Some(column)) => format!("{file}:{line}:{column}: "),
                (_, Some(line), Some(column)) => format!("line {line}, column {column}: "),
                _ => String::new(),
            };
            bail!("renderer failed: {location}{message}")
        }
        _ => Err(anyhow!(
            "renderer exited unsuccessfully: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
    }
}

fn extract_worker() -> Result<PathBuf> {
    let directories = ProjectDirs::from("dev", "Brume", "Brume")
        .ok_or_else(|| anyhow!("cannot resolve the Brume cache directory"))?;
    let digest = hex::encode(Sha256::digest(RENDERER_WORKER));
    let directory = directories.cache_dir().join("renderer").join(&digest);
    fs::create_dir_all(&directory)?;
    let lock_path = directory.join("extract.lock");
    let lock = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)?;
    lock.lock_exclusive()?;

    let file_name = if cfg!(windows) {
        "brume-renderer.exe"
    } else {
        "brume-renderer"
    };
    let destination = directory.join(file_name);
    if !destination.exists() {
        let temporary = directory.join(format!("{file_name}.tmp"));
        let mut file = fs::File::create(&temporary)?;
        file.write_all(RENDERER_WORKER)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporary, fs::Permissions::from_mode(0o700))?;
        }
        fs::rename(temporary, &destination)?;
    }
    FileExt::unlock(&lock)?;
    Ok(destination)
}
