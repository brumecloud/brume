use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
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
