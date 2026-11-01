//! Embeds the panel: every file of `web/dist` if it holds an `index.html`, else a page that
//! says how to build it. Writes `$OUT_DIR/panel.rs`, which `src/static_files.rs` includes.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

const PLACEHOLDER: &str = "<!doctype html>
<html lang=\"en\">
  <head>
    <meta charset=\"utf-8\">
    <title>Brume</title>
  </head>
  <body>
    <p>Build the panel: cd web &amp;&amp; ./node_modules/.bin/vp build</p>
  </body>
</html>
";

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("set by cargo"));
    let dist = manifest_dir.join("../../web/dist");
    // While `web/dist` is missing, cargo reruns this script on every build, so a later
    // `vp build` is picked up.
    println!("cargo::rerun-if-changed={}", dist.display());

    let mut files = Vec::new();
    if dist.join("index.html").is_file() {
        collect(&dist, &dist, &mut files)?;
    } else {
        let placeholder = out_dir.join("placeholder");
        fs::create_dir_all(&placeholder)?;
        let index = placeholder.join("index.html");
        fs::write(&index, PLACEHOLDER)?;
        files.push(("index.html".to_owned(), index));
    }
    files.sort();

    let mut source = String::from("/// The panel's files: path under `web/dist`, contents.\n");
    source.push_str("static PANEL_FILES: &[(&str, &[u8])] = &[\n");
    for (name, path) in &files {
        let path = path.canonicalize()?;
        let path = path.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "a panel path is not UTF-8")
        })?;
        writeln!(source, "    ({name:?}, include_bytes!({path:?})),").expect("writes to a String");
    }
    source.push_str("];\n");
    fs::write(out_dir.join("panel.rs"), source)
}

/// Adds the files under `dir` with their path relative to `root`, `/`-separated.
fn collect(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(root, &path, files)?;
        } else if kind.is_file() {
            let relative = path.strip_prefix(root).map_err(io::Error::other)?;
            let name: Vec<String> = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect();
            files.push((name.join("/"), path));
        }
    }
    Ok(())
}
