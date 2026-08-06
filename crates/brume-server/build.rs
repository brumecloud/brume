use std::{env, fs, path::PathBuf};

use anyhow::{Result, bail};

#[path = "../build_metadata.rs"]
mod build_metadata;

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let repository = manifest_dir.join("../..");
    let output = PathBuf::from(env::var("OUT_DIR")?);
    let metadata = build_metadata::collect(&repository);
    build_metadata::write_rust_source(&output, &metadata)?;

    let dist = env::var("BRUME_RENDERER_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../../renderer/dist"));
    let runtime = dist.join("web/runtime.js");
    let theme = dist.join("web/theme.css");
    let geist = dist.join("web/geist.woff2");
    let overlay = dist.join("web/overlay.js");
    for path in [&runtime, &theme, &geist, &overlay] {
        if !path.is_file() {
            bail!(
                "missing renderer web artifact {}; run the renderer web build first",
                path.display()
            );
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }
    fs::copy(runtime, output.join("runtime.js"))?;
    fs::copy(theme, output.join("theme.css"))?;
    fs::copy(geist, output.join("geist.woff2"))?;
    fs::copy(overlay, output.join("overlay.js"))?;
    Ok(())
}
