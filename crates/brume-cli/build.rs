use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let repository = manifest_dir.join("../..");
    println!("cargo:rerun-if-env-changed=BRUME_BUILD_COMMIT");
    println!(
        "cargo:rustc-env=BRUME_BUILD_COMMIT={}",
        build_commit(&repository)
    );

    let dist = env::var("BRUME_RENDERER_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../../renderer/dist"));
    let target = env::var("TARGET")?;
    let worker_name = if target.contains("windows") {
        "brume-renderer.exe"
    } else {
        "brume-renderer"
    };
    let worker = dist.join(&target).join(worker_name);
    let runtime = dist.join("web/runtime.js");
    let theme = dist.join("web/theme.css");
    for path in [&worker, &runtime, &theme] {
        if !path.is_file() {
            bail!(
                "missing renderer artifact {}; run ./scripts/build-renderer.sh {} first",
                path.display(),
                target
            );
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let output = PathBuf::from(env::var("OUT_DIR")?);
    fs::copy(&worker, output.join("brume-renderer"))
        .with_context(|| format!("copying {}", worker.display()))?;
    fs::copy(&runtime, output.join("runtime.js"))?;
    fs::copy(&theme, output.join("theme.css"))?;
    Ok(())
}

fn build_commit(repository: &Path) -> String {
    if let Some(commit) = env::var("BRUME_BUILD_COMMIT")
        .ok()
        .filter(|commit| !commit.is_empty())
    {
        return short_commit(&commit);
    }

    track_git_head(repository);
    git(repository, &["rev-parse", "--short=7", "HEAD"])
        .filter(|commit| !commit.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn short_commit(commit: &str) -> String {
    commit.chars().take(7).collect()
}

fn track_git_head(repository: &Path) {
    let Some(head) = git(repository, &["rev-parse", "--git-path", "HEAD"]) else {
        return;
    };
    println!("cargo:rerun-if-changed={}", repository.join(head).display());

    let Some(reference) = git(repository, &["symbolic-ref", "-q", "HEAD"]) else {
        return;
    };
    if let Some(reference_path) = git(repository, &["rev-parse", "--git-path", &reference]) {
        println!(
            "cargo:rerun-if-changed={}",
            repository.join(reference_path).display()
        );
    }
}

fn git(repository: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
