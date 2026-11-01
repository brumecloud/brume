//! Requests to the engine on `admin.sock`, from commands run as root on the host, such as
//! `brume login`.

use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use brume_proto::admin::{AdminReply, AdminRequest};
use brume_proto::frame::{FrameDecoder, encode};
use brume_runtime::paths::Paths;

/// The longest wait for the engine's answer.
const TIMEOUT: Duration = Duration::from_secs(10);

/// The size of each read from the socket.
const READ_CHUNK: usize = 4096;

/// Sends `request` to the engine on `admin.sock` and waits for its answer, 10 s at most.
/// Blocking. The engine closes the connection without an answer when it refuses, for instance
/// when it is read-only, and at once when the user is neither root nor the engine's.
pub fn admin_request(paths: &Paths, request: AdminRequest) -> anyhow::Result<AdminReply> {
    request_on(&paths.admin_socket(), &request, TIMEOUT)
}

fn request_on(
    socket: &Path,
    request: &AdminRequest,
    timeout: Duration,
) -> anyhow::Result<AdminReply> {
    let deadline = Instant::now() + timeout;
    let mut stream = UnixStream::connect(socket).map_err(|error| {
        let hint = match error.kind() {
            ErrorKind::PermissionDenied => " Run this command as root.",
            ErrorKind::NotFound | ErrorKind::ConnectionRefused => " Is the engine running?",
            _ => "",
        };
        anyhow::Error::new(error).context(format!("Cannot connect to {}.{hint}", socket.display()))
    })?;
    let no_answer = || format!("The engine did not answer within {timeout:?}.");
    stream
        .set_write_timeout(Some(timeout))
        .context("Cannot set a timeout on the socket.")?;
    stream
        .write_all(&encode(request)?)
        .context("Cannot send the request to the engine.")?;
    let mut decoder = FrameDecoder::default();
    let mut chunk = [0; READ_CHUNK];
    loop {
        if let Some(reply) = decoder
            .next::<AdminReply>()
            .context("The engine's answer is not one this command knows.")?
        {
            return Ok(reply);
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            bail!(no_answer());
        }
        stream
            .set_read_timeout(Some(left))
            .context("Cannot set a timeout on the socket.")?;
        let read = match stream.read(&mut chunk) {
            Ok(read) => read,
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                bail!(no_answer());
            }
            Err(error) => return Err(error).context("Cannot read the engine's answer."),
        };
        if read == 0 {
            bail!("The engine refused the request. Check its log.");
        }
        decoder.push(&chunk[..read]);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read as _, Write as _};
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use brume_proto::admin::{AdminReply, AdminRequest};
    use brume_proto::frame::{FrameDecoder, encode};

    use super::request_on;

    /// A directory under `/tmp`: macOS limits a unix socket path to 104 bytes.
    fn dir() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("brume-admin")
            .tempdir_in("/tmp")
            .expect("a temporary directory")
    }

    /// A fake engine on `admin.sock` that reads one request, then does `then` with the stream.
    fn engine(
        dir: &Path,
        then: impl FnOnce(std::os::unix::net::UnixStream) + Send + 'static,
    ) -> PathBuf {
        let socket = dir.join("admin.sock");
        let listener = UnixListener::bind(&socket).expect("binds");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accepts");
            let mut decoder = FrameDecoder::default();
            let mut chunk = [0; 1024];
            loop {
                if let Some(request) = decoder.next::<AdminRequest>().expect("decodes") {
                    assert_eq!(request, AdminRequest::CreateLoginLink);
                    break;
                }
                let read = stream.read(&mut chunk).expect("reads");
                decoder.push(&chunk[..read]);
            }
            then(stream);
        });
        socket
    }

    #[test]
    fn returns_the_engines_answer() {
        let dir = dir();
        let socket = engine(dir.path(), |mut stream| {
            let reply = AdminReply::LoginLink {
                token: "tok".to_owned(),
                expires_at: "2026-11-01T12:15:00Z".to_owned(),
            };
            stream
                .write_all(&encode(&reply).expect("encodes"))
                .expect("writes");
        });
        let reply = request_on(
            &socket,
            &AdminRequest::CreateLoginLink,
            Duration::from_secs(10),
        )
        .expect("an answer");
        assert!(matches!(reply, AdminReply::LoginLink { token, .. } if token == "tok"));
    }

    #[test]
    fn an_engine_that_closes_admin_sock_refuses() {
        let dir = dir();
        let socket = engine(dir.path(), drop);
        let error = request_on(
            &socket,
            &AdminRequest::CreateLoginLink,
            Duration::from_secs(10),
        )
        .expect_err("refused");
        assert_eq!(
            error.to_string(),
            "The engine refused the request. Check its log."
        );
    }

    #[test]
    fn an_engine_that_never_answers_times_out() {
        let dir = dir();
        let socket = engine(dir.path(), |stream| {
            std::thread::sleep(Duration::from_secs(2));
            drop(stream);
        });
        let error = request_on(
            &socket,
            &AdminRequest::CreateLoginLink,
            Duration::from_millis(200),
        )
        .expect_err("no answer");
        assert_eq!(error.to_string(), "The engine did not answer within 200ms.");
    }

    #[test]
    fn without_a_socket_asks_whether_the_engine_runs() {
        let error = request_on(
            Path::new("/nonexistent/admin.sock"),
            &AdminRequest::CreateLoginLink,
            Duration::from_secs(10),
        )
        .expect_err("no engine");
        assert_eq!(
            error.to_string(),
            "Cannot connect to /nonexistent/admin.sock. Is the engine running?"
        );
    }
}
