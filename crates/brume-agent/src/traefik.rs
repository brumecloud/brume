//! Traefik's config on the host: the agent writes it, Traefik reads it through a read-only
//! mount and reloads it when it changes. It holds hosts and service names, never a secret, so it
//! is readable by all: Traefik runs as nobody, remapped by `userns-remap`.

use std::fs::{DirBuilder, OpenOptions, Permissions};
use std::io::{ErrorKind, Write as _};
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::path::Path;

use brume_agent_core::ingress::TRAEFIK_CONFIG_FILE;
use serde_json::Value;

/// The mode of the config's directory.
const DIR_MODE: u32 = 0o755;
/// The mode of the config file.
const FILE_MODE: u32 = 0o644;
/// The file being written, beside the config: Traefik loads `.yml`, `.yaml` and `.toml` files
/// only, and watches the config file's name.
const TEMP_FILE: &str = "dynamic.yml.tmp";

/// Writes `config` to `dir/dynamic.yml` unless it already holds it, and whether it wrote. The
/// directory is created if missing, in `0755`; the file is replaced at once, by a rename, in
/// `0644`, so that Traefik never reads half a file.
pub fn write_config(dir: &Path, config: &Value) -> std::io::Result<bool> {
    let mut content = serde_json::to_vec_pretty(config).map_err(std::io::Error::other)?;
    content.push(b'\n');
    let path = dir.join(TRAEFIK_CONFIG_FILE);
    match std::fs::read(&path) {
        Ok(current) if current == content => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    create_dir(dir)?;
    let temp = dir.join(TEMP_FILE);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(FILE_MODE)
        .open(&temp)?;
    // The umask may have taken bits off.
    file.set_permissions(Permissions::from_mode(FILE_MODE))?;
    file.write_all(&content)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temp, &path)?;
    Ok(true)
}

/// Creates `dir` and its parents if missing, in `0755`. An existing directory keeps its mode:
/// on a host, `brume install` made it.
fn create_dir(dir: &Path) -> std::io::Result<()> {
    if dir.is_dir() {
        return Ok(());
    }
    DirBuilder::new()
        .recursive(true)
        .mode(DIR_MODE)
        .create(dir)?;
    std::fs::set_permissions(dir, Permissions::from_mode(DIR_MODE))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use serde_json::json;

    use super::write_config;

    fn mode(path: &std::path::Path) -> u32 {
        std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777
    }

    #[test]
    fn writes_the_config_once_readable_by_all() {
        let root = tempfile::tempdir().expect("a temporary directory");
        let dir = root.path().join("agent/traefik");
        let config = json!({});

        assert!(write_config(&dir, &config).expect("writes"));
        assert_eq!(mode(&dir), 0o755);
        let file = dir.join("dynamic.yml");
        assert_eq!(mode(&file), 0o644);
        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).expect("reads")).expect("JSON");
        assert_eq!(written, config);
        // Only the config: the temporary file is gone.
        let names: Vec<String> = std::fs::read_dir(&dir)
            .expect("lists")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(names, ["dynamic.yml"]);

        // The same config again: nothing written, so Traefik reloads nothing.
        assert!(!write_config(&dir, &config).expect("checks"));

        // Another one replaces it, with the same mode.
        let routed = json!({ "http": { "routers": { "a": {} }, "services": {} } });
        assert!(write_config(&dir, &routed).expect("rewrites"));
        assert_eq!(mode(&file), 0o644);
        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).expect("reads")).expect("JSON");
        assert_eq!(written, routed);
    }

    #[test]
    fn an_existing_directory_keeps_its_mode() {
        let root = tempfile::tempdir().expect("a temporary directory");
        let dir = root.path().join("traefik");
        std::fs::create_dir(&dir).expect("creates");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o750)).expect("chmod");
        assert!(write_config(&dir, &json!({})).expect("writes"));
        assert_eq!(mode(&dir), 0o750);
        assert_eq!(mode(&dir.join("dynamic.yml")), 0o644);
    }
}
