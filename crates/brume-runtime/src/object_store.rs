//! Object storage: where the engine ships its log. A local directory in this milestone; S3 will
//! be a second implementation of the same trait. Objects are written once, never overwritten.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The longest key, in bytes.
const MAX_KEY_LEN: usize = 1024;

/// What `put_if_absent` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PutOutcome {
    Created,
    /// An object already has this key; it was left as it is.
    AlreadyExists,
}

/// A store of immutable objects, by key.
pub trait ObjectStore {
    /// Writes the object unless one already has this key.
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> io::Result<PutOutcome>;

    /// The object with this key.
    fn get(&self, key: &str) -> io::Result<Option<Vec<u8>>>;

    /// The keys that start with `prefix`, sorted.
    fn list(&self, prefix: &str) -> io::Result<Vec<String>>;
}

/// An object store in a local directory: a key is a relative path.
#[derive(Clone, Debug)]
pub struct DirStore {
    root: PathBuf,
}

/// Refuses a key that could leave the root or that is not plain: only `[a-z0-9/._-]`, no `..`,
/// no leading `/`, no empty segment, and no segment starting with `.` (temporary files).
fn check_key(key: &str, allow_trailing_slash: bool) -> io::Result<()> {
    let refuse = || {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "an object key has only a-z, 0-9, /, ., _ and -, without .. or empty segments",
        ))
    };
    let allowed = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b"/._-".contains(&b);
    if key.len() > MAX_KEY_LEN || !key.bytes().all(allowed) || key.contains("..") {
        return refuse();
    }
    let body = match key.strip_suffix('/') {
        Some(body) if allow_trailing_slash => body,
        _ => key,
    };
    if body.is_empty() && !(allow_trailing_slash && key.is_empty()) {
        return refuse();
    }
    if !body.is_empty()
        && body
            .split('/')
            .any(|segment| segment.is_empty() || segment.starts_with('.'))
    {
        return refuse();
    }
    Ok(())
}

impl DirStore {
    /// A store whose objects live under `root`.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path(&self, key: &str) -> io::Result<PathBuf> {
        check_key(key, false)?;
        Ok(self.root.join(key))
    }

    fn collect(&self, dir: &Path, prefix: &str, keys: &mut Vec<String>) -> io::Result<()> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        for entry in entries {
            let entry = entry?;
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let key = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if entry.file_type()?.is_dir() {
                self.collect(&entry.path(), &key, keys)?;
            } else {
                keys.push(key);
            }
        }
        Ok(())
    }
}

impl ObjectStore for DirStore {
    /// Writes a temporary file, syncs it, then hard-links it to the key: the link fails if the
    /// key exists, so an object is never overwritten, even by a concurrent writer.
    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> io::Result<PutOutcome> {
        let path = self.path(key)?;
        let dir = path.parent().unwrap_or(&self.root).to_path_buf();
        std::fs::create_dir_all(&dir)?;
        let mut suffix = [0u8; 16];
        rand::fill(&mut suffix);
        let temporary = dir.join(format!(".tmp-{}", hex_encode(&suffix)));
        let written = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()
        })();
        let linked = written.and_then(|()| std::fs::hard_link(&temporary, &path));
        let removed = std::fs::remove_file(&temporary);
        match linked {
            Ok(()) => {
                removed?;
                File::open(&dir)?.sync_all()?;
                Ok(PutOutcome::Created)
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                Ok(PutOutcome::AlreadyExists)
            }
            Err(error) => Err(error),
        }
    }

    fn get(&self, key: &str) -> io::Result<Option<Vec<u8>>> {
        match std::fs::read(self.path(key)?) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn list(&self, prefix: &str) -> io::Result<Vec<String>> {
        check_key(prefix, true)?;
        let mut keys = Vec::new();
        self.collect(&self.root, "", &mut keys)?;
        keys.retain(|key| key.starts_with(prefix));
        keys.sort();
        Ok(keys)
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
