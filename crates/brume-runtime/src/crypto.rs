//! Sealing with the master key: ChaCha20-Poly1305, a random 12-byte nonce in front of the
//! ciphertext. The associated data binds a sealed value to its row, so that one cannot be
//! moved to another.

use std::fs::{DirBuilder, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use secrecy::{ExposeSecret, SecretBox};

/// The length of the master key, in bytes.
pub const KEY_LEN: usize = 32;

/// The length of the nonce in front of a sealed value.
const NONCE_LEN: usize = 12;

/// The key that seals secrets at rest. Its `Debug` hides it.
pub struct MasterKey(SecretBox<[u8; KEY_LEN]>);

impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MasterKey([REDACTED])")
    }
}

/// A sealed value that does not open: the wrong key, the wrong row, or altered bytes.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("a sealed value does not open")]
pub struct OpenError;

impl MasterKey {
    /// Reads the key at `path`, or creates it: 32 random bytes, readable by the owner only. A
    /// key that others can read is refused.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        match std::fs::File::open(path) {
            Ok(mut file) => {
                if file.metadata()?.permissions().mode() & 0o077 != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "the master key must be readable by its owner only (0600)",
                    ));
                }
                let mut bytes = [0u8; KEY_LEN];
                file.read_exact(&mut bytes)?;
                let mut rest = Vec::new();
                file.read_to_end(&mut rest)?;
                if !rest.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "the master key must be 32 bytes",
                    ));
                }
                Ok(Self::from_bytes(bytes))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if let Some(parent) = path.parent() {
                    DirBuilder::new()
                        .recursive(true)
                        .mode(0o700)
                        .create(parent)?;
                }
                let mut bytes = [0u8; KEY_LEN];
                rand::fill(&mut bytes);
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(path)?;
                file.write_all(&bytes)?;
                file.sync_all()?;
                Ok(Self::from_bytes(bytes))
            }
            Err(error) => Err(error),
        }
    }

    /// A key from its bytes.
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(SecretBox::new(Box::new(bytes)))
    }

    fn cipher(&self) -> ChaCha20Poly1305 {
        ChaCha20Poly1305::new(&Key::from(*self.0.expose_secret()))
    }
}

/// Seals `plaintext`, bound to `aad`: the nonce, then the ciphertext and its tag.
pub fn seal(key: &MasterKey, aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let mut nonce = [0u8; NONCE_LEN];
    rand::fill(&mut nonce);
    let ciphertext = key
        .cipher()
        .encrypt(
            &Nonce::from(nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("ChaCha20-Poly1305 seals any message that fits in memory");
    let mut sealed = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    sealed.extend_from_slice(&nonce);
    sealed.extend_from_slice(&ciphertext);
    sealed
}

/// Opens what `seal` sealed with the same key and `aad`.
pub fn open(key: &MasterKey, aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, OpenError> {
    let Some((nonce, ciphertext)) = sealed.split_first_chunk::<NONCE_LEN>() else {
        return Err(OpenError);
    };
    key.cipher()
        .decrypt(
            &Nonce::from(*nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| OpenError)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::{MasterKey, OpenError, open, seal};

    #[test]
    fn sealed_value_opens_only_with_its_key_and_row() {
        let key = MasterKey::from_bytes([7; 32]);
        let sealed = seal(&key, b"secret:a:1", "s3cr3t-🚀".as_bytes());
        assert!(!sealed.windows(6).any(|window| window == b"s3cr3t"));
        assert_ne!(
            seal(&key, b"secret:a:1", b"x"),
            seal(&key, b"secret:a:1", b"x")
        );
        assert_eq!(
            open(&key, b"secret:a:1", &sealed).as_deref(),
            Ok("s3cr3t-🚀".as_bytes())
        );
        assert_eq!(open(&key, b"secret:a:2", &sealed), Err(OpenError));
        assert_eq!(
            open(&MasterKey::from_bytes([8; 32]), b"secret:a:1", &sealed),
            Err(OpenError)
        );
        let mut altered = sealed.clone();
        altered[20] ^= 1;
        assert_eq!(open(&key, b"secret:a:1", &altered), Err(OpenError));
        assert_eq!(open(&key, b"secret:a:1", &sealed[..5]), Err(OpenError));
        assert!(!format!("{key:?}").contains('7'));
    }

    #[test]
    fn key_file_is_created_once_with_mode_0600() {
        let home = tempfile::tempdir().expect("a temporary directory");
        let path = home.path().join("engine/master.key");
        let created = MasterKey::load_or_create(&path).expect("creates");
        let mode = std::fs::metadata(&path)
            .expect("exists")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        assert_eq!(std::fs::read(&path).expect("reads").len(), 32);
        let loaded = MasterKey::load_or_create(&path).expect("loads");
        let sealed = seal(&created, b"row", b"value");
        assert_eq!(open(&loaded, b"row", &sealed).as_deref(), Ok(&b"value"[..]));

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");
        assert_eq!(
            MasterKey::load_or_create(&path)
                .expect_err("refused")
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }
}
