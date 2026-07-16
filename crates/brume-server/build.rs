use std::{env, fs, path::PathBuf};

use anyhow::{Result, bail};

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let dist = env::var("BRUME_RENDERER_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../../renderer/dist"));
    let runtime = dist.join("web/runtime.js");
    let theme = dist.join("web/theme.css");
    for path in [&runtime, &theme] {
        if !path.is_file() {
            bail!(
                "missing renderer web artifact {}; run the renderer web build first",
                path.display()
            );
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let output = PathBuf::from(env::var("OUT_DIR")?);
    fs::copy(runtime, output.join("runtime.js"))?;
    fs::copy(theme, output.join("theme.css"))?;
    Ok(())
}
