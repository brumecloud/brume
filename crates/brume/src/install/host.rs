//! The host that `brume install` configures: its commands and its files, behind the `System`
//! trait, so that the tests run the steps against a fake.

use std::fmt;
use std::fs::{OpenOptions, Permissions};
use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::fs::{
    DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _,
};
use std::path::Path;
use std::process::Stdio;

use anyhow::{Context as _, bail};
use brume_proto::admin::{AdminIngress, AdminReply, AdminRequest};
use brume_runtime::paths::Paths;

use super::cloudflare::TOKEN_ENV;

/// The largest file that `brume install` reads: `/etc/os-release`, and the files it wrote.
const MAX_READ_BYTES: u64 = 1024 * 1024;

/// The most of a failed command's output that an error quotes.
const MAX_QUOTED_BYTES: usize = 2048;

/// The exit code of a program that is not installed, as a shell reports it.
const NOT_FOUND: i32 = 127;

/// A command run on the host: a program and its arguments, never through a shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cmd {
    program: &'static str,
    args: Vec<String>,
    env: Vec<(&'static str, &'static str)>,
}

impl Cmd {
    pub fn new(program: &'static str, args: &[&str]) -> Self {
        Self {
            program,
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            env: Vec::new(),
        }
    }

    /// Sets the variable `key` for the command.
    pub fn env(mut self, key: &'static str, value: &'static str) -> Self {
        self.env.push((key, value));
        self
    }
}

/// The command line, as the install prints it in an error and as the fake host keys it.
impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.program)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

/// What a command did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The exit code; `None` if a signal killed the command.
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }

    /// A successful command that printed `stdout`.
    #[cfg(test)]
    pub fn ok(stdout: &str) -> Self {
        Self {
            code: Some(0),
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    /// A command that failed with `code`.
    #[cfg(test)]
    pub fn failed(code: i32) -> Self {
        Self {
            code: Some(code),
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

/// Whether a command only reads the host, or changes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Reads,
    Changes,
}

/// The owner, group and mode of a directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirMeta {
    pub user: String,
    pub group: String,
    /// The permission bits, setgid and sticky included, such as `0o3770`.
    pub mode: u32,
}

impl DirMeta {
    pub fn new(user: &str, group: &str, mode: u32) -> Self {
        Self {
            user: user.to_owned(),
            group: group.to_owned(),
            mode,
        }
    }
}

/// The operations of a host. `RealSystem` does them, a fake records them in the tests.
pub trait System {
    /// Runs `command` to the end. A program that is not installed exits with 127, as in a
    /// shell; any other failure to start it is an error.
    fn run(&self, command: &Cmd, effect: Effect) -> anyhow::Result<Output>;
    /// The contents of the file at `path`, or `None` if it is missing.
    fn read(&self, path: &Path) -> anyhow::Result<Option<String>>;
    /// Replaces the file at `path` atomically, owned by root with `mode`, creating its missing
    /// parent directories (`0755`).
    fn write(&self, path: &Path, contents: &str, mode: u32) -> anyhow::Result<()>;
    /// The owner, group and mode of the directory at `path`, or `None` if it is missing.
    fn dir(&self, path: &Path) -> anyhow::Result<Option<DirMeta>>;
    /// Whether something exists at `path`.
    fn exists(&self, path: &Path) -> bool;
    /// Sends `request` to the engine on `admin.sock`. A refusal is an `admin::Refused` error.
    fn admin(&self, request: &AdminRequest, effect: Effect) -> anyhow::Result<AdminReply>;
}

/// The host, as the steps see it. A step's `is_done` gets a `&Host`, which only reads; its
/// `apply` gets a `&mut Host`, which also changes the host.
pub struct Host {
    system: Box<dyn System>,
}

impl Host {
    pub fn new(system: Box<dyn System>) -> Self {
        Self { system }
    }

    /// This machine.
    pub fn real() -> Self {
        Self::new(Box::new(RealSystem))
    }

    /// Runs `command`, which only reads the host. Its exit code is part of the answer: a failure
    /// is not an error.
    pub fn query(&self, command: &Cmd) -> anyhow::Result<Output> {
        self.system.run(command, Effect::Reads)
    }

    /// The trimmed standard output of `command` if it succeeds, else `None`.
    pub fn query_stdout(&self, command: &Cmd) -> anyhow::Result<Option<String>> {
        let output = self.query(command)?;
        Ok(output.success().then(|| output.stdout.trim().to_owned()))
    }

    /// Runs `command`, which changes the host, and returns its standard output. Fails unless the
    /// command succeeds, quoting the end of what it printed.
    pub fn change(&mut self, command: &Cmd) -> anyhow::Result<String> {
        let output = self.system.run(command, Effect::Changes)?;
        if output.success() {
            return Ok(output.stdout);
        }
        let status = match output.code {
            Some(code) => format!("exited with {code}"),
            None => "was killed".to_owned(),
        };
        let printed = format!("{}{}", output.stdout, output.stderr);
        bail!(
            "`{command}` {status}{}",
            match tail(printed.trim()) {
                "" => String::new(),
                printed => format!(":\n{printed}"),
            }
        )
    }

    /// Runs `command`, which changes the host, and returns what it did, success or not: the
    /// caller reads its output either way.
    pub fn attempt(&mut self, command: &Cmd) -> anyhow::Result<Output> {
        self.system.run(command, Effect::Changes)
    }

    /// The engine's public entry. Fails if the engine does not answer.
    pub fn engine_ingress(&self) -> anyhow::Result<Option<AdminIngress>> {
        match self
            .system
            .admin(&AdminRequest::GetIngress, Effect::Reads)?
        {
            AdminReply::Ingress(ingress) => Ok(ingress),
            _ => bail!("The engine answered with something other than its public entry."),
        }
    }

    /// Sends `request`, which changes the engine's state, on `admin.sock`.
    pub fn admin_change(&mut self, request: &AdminRequest) -> anyhow::Result<AdminReply> {
        self.system.admin(request, Effect::Changes)
    }

    /// The contents of the file at `path`, or `None` if it is missing.
    pub fn read(&self, path: &Path) -> anyhow::Result<Option<String>> {
        self.system.read(path)
    }

    /// Whether the file at `path` holds exactly `contents`.
    pub fn has_file(&self, path: &Path, contents: &str) -> anyhow::Result<bool> {
        Ok(self.read(path)?.as_deref() == Some(contents))
    }

    /// Replaces the file at `path` atomically, owned by root with `mode`.
    pub fn write(&mut self, path: &Path, contents: &str, mode: u32) -> anyhow::Result<()> {
        self.system.write(path, contents, mode)
    }

    /// The owner, group and mode of the directory at `path`, or `None` if it is missing.
    pub fn dir(&self, path: &Path) -> anyhow::Result<Option<DirMeta>> {
        self.system.dir(path)
    }

    /// Whether something exists at `path`.
    pub fn exists(&self, path: &Path) -> bool {
        self.system.exists(path)
    }
}

/// The last `MAX_QUOTED_BYTES` of `text`, on a character boundary.
pub(super) fn tail(text: &str) -> &str {
    let mut start = text.len().saturating_sub(MAX_QUOTED_BYTES);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    &text[start..]
}

/// This machine.
pub struct RealSystem;

impl System for RealSystem {
    fn run(&self, command: &Cmd, _effect: Effect) -> anyhow::Result<Output> {
        let result = std::process::Command::new(command.program)
            .args(&command.args)
            // No command needs the Cloudflare token: none inherits it.
            .env_remove(TOKEN_ENV)
            .envs(command.env.iter().copied())
            .stdin(Stdio::null())
            .output();
        let output = match result {
            Ok(output) => output,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(Output {
                    code: Some(NOT_FOUND),
                    stdout: String::new(),
                    stderr: format!("{}: command not found", command.program),
                });
            }
            Err(error) => {
                return Err(error).with_context(|| format!("cannot run `{command}`"));
            }
        };
        Ok(Output {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    fn read(&self, path: &Path) -> anyhow::Result<Option<String>> {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(error).with_context(|| format!("cannot open {}", path.display()));
            }
        };
        let mut bytes = Vec::new();
        file.take(MAX_READ_BYTES + 1)
            .read_to_end(&mut bytes)
            .with_context(|| format!("cannot read {}", path.display()))?;
        if bytes.len() as u64 > MAX_READ_BYTES {
            bail!("{} is larger than 1 MiB", path.display());
        }
        // A file that is not UTF-8 is not one that `brume install` wrote.
        Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn write(&self, path: &Path, contents: &str, mode: u32) -> anyhow::Result<()> {
        let parent = path
            .parent()
            .with_context(|| format!("{} has no parent directory", path.display()))?;
        let name = path
            .file_name()
            .with_context(|| format!("{} has no file name", path.display()))?;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o755)
            .create(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
        let temporary = parent.join(format!(".{}.brume-install", name.to_string_lossy()));
        match std::fs::remove_file(&temporary) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("cannot remove {}", temporary.display()));
            }
        }
        let written = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(mode)
                .open(&temporary)?;
            // The umask may have cleared bits of `mode`.
            file.set_permissions(Permissions::from_mode(mode))?;
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            std::fs::rename(&temporary, path)?;
            std::fs::File::open(parent)?.sync_all()
        })();
        if written.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        written.with_context(|| format!("cannot write {}", path.display()))
    }

    fn dir(&self, path: &Path) -> anyhow::Result<Option<DirMeta>> {
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(error).with_context(|| format!("cannot read {}", path.display()));
            }
        };
        if !metadata.is_dir() {
            bail!("{} is not a directory", path.display());
        }
        let passwd = self.read(Path::new("/etc/passwd"))?.unwrap_or_default();
        let group = self.read(Path::new("/etc/group"))?.unwrap_or_default();
        Ok(Some(DirMeta {
            user: name_of(&passwd, metadata.uid()),
            group: name_of(&group, metadata.gid()),
            mode: metadata.mode() & 0o7777,
        }))
    }

    fn exists(&self, path: &Path) -> bool {
        path.symlink_metadata().is_ok()
    }

    fn admin(&self, request: &AdminRequest, _effect: Effect) -> anyhow::Result<AdminReply> {
        crate::admin::admin_request(&Paths::system(), request)
    }
}

/// The name of the first entry of `/etc/passwd` or `/etc/group` (`contents`) whose id is `id`,
/// or the id itself.
fn name_of(contents: &str, id: u32) -> String {
    let id = id.to_string();
    contents
        .lines()
        .map(|line| line.split(':').collect::<Vec<_>>())
        .find(|fields| fields.get(2) == Some(&id.as_str()))
        .and_then(|fields| fields.first().map(|name| (*name).to_owned()))
        .unwrap_or(id)
}

#[cfg(test)]
pub mod fake {
    //! A host in memory: it answers the queries it was told to, keeps the files written, and
    //! records every change.

    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    use anyhow::bail;
    use brume_proto::admin::{AdminIngress, AdminReply, AdminRequest};
    use secrecy::ExposeSecret as _;

    use super::{Cmd, DirMeta, Effect, Output, System};
    use crate::admin::Refused;

    /// An answer to a query: given once `after` is among the changes, if set. Each query takes
    /// the first output, the last one stays.
    struct Answer {
        query: String,
        after: Option<String>,
        outputs: Vec<Output>,
    }

    #[derive(Default)]
    struct State {
        files: BTreeMap<PathBuf, String>,
        dirs: BTreeMap<PathBuf, DirMeta>,
        paths: BTreeSet<PathBuf>,
        answers: Vec<Answer>,
        changes: Vec<String>,
        engine: Option<FakeEngine>,
    }

    /// The engine on `admin.sock`, once started.
    #[derive(Default)]
    struct FakeEngine {
        ingress: Option<AdminIngress>,
        tunnel_token: Option<String>,
        refusal: Option<String>,
    }

    /// A fake host. Its clones share their state: the test keeps one, the `Host` another.
    #[derive(Clone, Default)]
    pub struct FakeSystem(Rc<RefCell<State>>);

    impl FakeSystem {
        /// `query` answers `output`. A later answer to the same query wins.
        pub fn answer(&self, query: &Cmd, output: Output) {
            self.0.borrow_mut().answers.push(Answer {
                query: query.to_string(),
                after: None,
                outputs: vec![output],
            });
        }

        /// `query` answers each of `outputs` in turn, then the last one.
        pub fn answer_each(&self, query: &Cmd, outputs: Vec<Output>) {
            assert!(!outputs.is_empty(), "an answer");
            self.0.borrow_mut().answers.push(Answer {
                query: query.to_string(),
                after: None,
                outputs,
            });
        }

        /// `query` answers `output` once the change `after` has run.
        pub fn answer_after(&self, after: &Cmd, query: &Cmd, output: Output) {
            self.answer_after_each(after, query, vec![output]);
        }

        /// `query` answers each of `outputs` in turn once the change `after` has run, then the
        /// last one.
        pub fn answer_after_each(&self, after: &Cmd, query: &Cmd, outputs: Vec<Output>) {
            assert!(!outputs.is_empty(), "an answer");
            self.0.borrow_mut().answers.push(Answer {
                query: query.to_string(),
                after: Some(after.to_string()),
                outputs,
            });
        }

        pub fn put_file(&self, path: &str, contents: &str) {
            self.0
                .borrow_mut()
                .files
                .insert(PathBuf::from(path), contents.to_owned());
        }

        pub fn put_dir(&self, path: &str, meta: DirMeta) {
            self.0.borrow_mut().dirs.insert(PathBuf::from(path), meta);
        }

        /// Something exists at `path`, such as a program.
        pub fn put_path(&self, path: &str) {
            self.0.borrow_mut().paths.insert(PathBuf::from(path));
        }

        pub fn file(&self, path: &str) -> Option<String> {
            self.0.borrow().files.get(Path::new(path)).cloned()
        }

        /// The changes so far, in order: each command that changes the host, and `write <path>`
        /// for each file written.
        pub fn changes(&self) -> Vec<String> {
            self.0.borrow().changes.clone()
        }

        pub fn clear_changes(&self) {
            self.0.borrow_mut().changes.clear();
        }

        /// The engine answers on `admin.sock`.
        pub fn start_engine(&self) {
            self.0
                .borrow_mut()
                .engine
                .get_or_insert_with(FakeEngine::default);
        }

        /// The engine refuses every request with `reason`.
        pub fn refuse_admin(&self, reason: &str) {
            self.start_engine();
            if let Some(engine) = &mut self.0.borrow_mut().engine {
                engine.refusal = Some(reason.to_owned());
            }
        }

        /// The engine's public entry, and the tunnel token it was given.
        pub fn engine_ingress(&self) -> Option<(AdminIngress, String)> {
            let state = self.0.borrow();
            let engine = state.engine.as_ref()?;
            Some((engine.ingress.clone()?, engine.tunnel_token.clone()?))
        }
    }

    impl System for FakeSystem {
        fn run(&self, command: &Cmd, effect: Effect) -> anyhow::Result<Output> {
            let line = command.to_string();
            let mut state = self.0.borrow_mut();
            if effect == Effect::Changes {
                state.changes.push(line.clone());
            }
            let State {
                answers, changes, ..
            } = &mut *state;
            let answer = answers.iter_mut().rev().find(|answer| {
                answer.query == line
                    && answer
                        .after
                        .as_ref()
                        .is_none_or(|after| changes.contains(after))
            });
            Ok(match (answer, effect) {
                (Some(answer), _) if answer.outputs.len() > 1 => answer.outputs.remove(0),
                (Some(answer), _) => answer.outputs[0].clone(),
                // A change succeeds, an unknown query fails.
                (None, Effect::Changes) => Output::ok(""),
                (None, Effect::Reads) => Output::failed(1),
            })
        }

        fn read(&self, path: &Path) -> anyhow::Result<Option<String>> {
            Ok(self.0.borrow().files.get(path).cloned())
        }

        fn write(&self, path: &Path, contents: &str, _mode: u32) -> anyhow::Result<()> {
            let mut state = self.0.borrow_mut();
            state.changes.push(format!("write {}", path.display()));
            state.files.insert(path.to_owned(), contents.to_owned());
            Ok(())
        }

        fn dir(&self, path: &Path) -> anyhow::Result<Option<DirMeta>> {
            Ok(self.0.borrow().dirs.get(path).cloned())
        }

        fn exists(&self, path: &Path) -> bool {
            let state = self.0.borrow();
            state.paths.contains(path)
                || state.files.contains_key(path)
                || state.dirs.contains_key(path)
        }

        /// Records each change as `admin <request>`, without its token.
        fn admin(&self, request: &AdminRequest, effect: Effect) -> anyhow::Result<AdminReply> {
            let mut state = self.0.borrow_mut();
            if effect == Effect::Changes {
                let line = match request {
                    AdminRequest::CreateLoginLink => "admin CreateLoginLink".to_owned(),
                    AdminRequest::GetIngress => "admin GetIngress".to_owned(),
                    AdminRequest::ConfigureIngress {
                        zone, tunnel_id, ..
                    } => format!("admin ConfigureIngress {zone} {tunnel_id}"),
                };
                state.changes.push(line);
            }
            let Some(engine) = &mut state.engine else {
                bail!("Cannot connect to /run/brume/engine/admin.sock. Is the engine running?");
            };
            if let Some(reason) = &engine.refusal {
                return Err(Refused {
                    reason: reason.clone(),
                }
                .into());
            }
            Ok(match request {
                AdminRequest::CreateLoginLink => AdminReply::LoginLink {
                    token: "login-token".to_owned(),
                    expires_at: "2026-11-01T12:15:00Z".to_owned(),
                },
                AdminRequest::GetIngress => AdminReply::Ingress(engine.ingress.clone()),
                AdminRequest::ConfigureIngress {
                    zone,
                    tunnel_id,
                    tunnel_token,
                } => {
                    engine.ingress = Some(AdminIngress {
                        zone: zone.clone(),
                        tunnel_id: tunnel_id.clone(),
                    });
                    engine.tunnel_token = Some(tunnel_token.expose_secret().to_owned());
                    AdminReply::IngressConfigured
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use super::{Cmd, Host, RealSystem, System as _, name_of, tail};

    #[test]
    fn a_failed_change_quotes_the_command_and_its_output() {
        let mut host = Host::real();
        let error = host
            .change(&Cmd::new("sh", &["-c", "echo out; echo err >&2; exit 3"]))
            .expect_err("fails");
        assert_eq!(
            error.to_string(),
            "`sh -c echo out; echo err >&2; exit 3` exited with 3:\nout\nerr"
        );
        let output = host
            .query(&Cmd::new("brume-no-such-program", &[]))
            .expect("not an error");
        assert_eq!(output.code, Some(127));
    }

    #[test]
    fn writes_atomically_with_the_mode() {
        let dir = tempfile::tempdir().expect("a temporary directory");
        let path = dir.path().join("etc/brume/api.toml");
        RealSystem.write(&path, "one", 0o600).expect("writes");
        RealSystem.write(&path, "two", 0o644).expect("replaces");
        assert_eq!(
            RealSystem.read(&path).expect("reads").as_deref(),
            Some("two")
        );
        let mode = std::fs::metadata(&path)
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o644);
        let names: Vec<_> = std::fs::read_dir(path.parent().expect("a parent"))
            .expect("lists")
            .map(|entry| entry.expect("an entry").file_name())
            .collect();
        assert_eq!(names, ["api.toml"]);
        assert_eq!(
            RealSystem.read(&dir.path().join("missing")).expect("reads"),
            None
        );
    }

    #[test]
    fn names_an_id_from_passwd() {
        let passwd = "root:x:0:0::/root:/bin/sh\nbrume-api:x:992:992::/nonexistent:/x\n";
        assert_eq!(name_of(passwd, 992), "brume-api");
        assert_eq!(name_of(passwd, 0), "root");
        assert_eq!(name_of(passwd, 5), "5");
    }

    #[test]
    fn quotes_the_end_of_a_long_output() {
        let long = format!("{}é{}", "a".repeat(10), "b".repeat(2047));
        assert_eq!(tail(&long), &long[long.len() - 2047..]);
    }
}
