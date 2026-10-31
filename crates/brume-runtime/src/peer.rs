//! Who is at the other end of a unix socket.

use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use tokio::net::UnixStream;

/// The user id of the process at the other end of `stream`.
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    Ok(stream.peer_cred()?.uid())
}

/// The user id that owns `path`: for the run directory, the only user the services trust.
pub fn owner_uid(path: &Path) -> io::Result<u32> {
    Ok(std::fs::metadata(path)?.uid())
}

#[cfg(test)]
mod tests {
    use super::{owner_uid, peer_uid};

    #[tokio::test]
    async fn peer_is_the_same_user_as_the_owner_of_its_files() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let (left, _right) = tokio::net::UnixStream::pair().expect("a socket pair");
        assert_eq!(
            peer_uid(&left).expect("credentials"),
            owner_uid(dir.path()).expect("metadata")
        );
    }
}
