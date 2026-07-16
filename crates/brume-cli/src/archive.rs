use std::{fs::File, path::Path};

use anyhow::{Context, Result, bail};
use tar::Builder;
use walkdir::WalkDir;

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
