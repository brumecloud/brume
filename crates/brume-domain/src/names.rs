//! Names and paths. Each one ends up in Docker, DNS, a URL or a process's argv, so each is
//! checked against an allowlist.

use crate::ValidationError;

/// The longest image reference, as Docker counts it.
const MAX_IMAGE_REF_LEN: usize = 255;

/// A project's name, as the admin typed it: 1 to 64 characters, no control character.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectName(String);

impl ProjectName {
    /// Checks the length and the absence of control characters.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        parse_display_name("project name", raw).map(Self)
    }
}

string_newtype!(ProjectName);

/// A service's name, as the admin typed it: 1 to 64 characters, no control character.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceName(String);

impl ServiceName {
    /// Checks the length and the absence of control characters.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        parse_display_name("service name", raw).map(Self)
    }
}

string_newtype!(ServiceName);

/// A DNS label, `^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$`. It names environments and gives each
/// service its host name on the environment network.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slug(String);

impl Slug {
    /// Checks the DNS label grammar.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let is_alphanumeric = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
        let bytes = raw.as_bytes();
        let valid = match (bytes.first(), bytes.last()) {
            (Some(&first), Some(&last)) => {
                bytes.len() <= 63
                    && is_alphanumeric(first)
                    && is_alphanumeric(last)
                    && bytes.iter().all(|&b| is_alphanumeric(b) || b == b'-')
            }
            _ => false,
        };
        if !valid {
            return Err(ValidationError::new(
                "slug",
                "must be 1 to 63 lowercase letters, digits or hyphens, and start and end with a \
                 letter or a digit",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(Slug);

/// An environment variable's name, `^[A-Za-z_][A-Za-z0-9_]{0,127}$`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarName(String);

impl VarName {
    /// Checks the identifier grammar.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let valid = match raw.as_bytes().split_first() {
            Some((&first, rest)) => {
                raw.len() <= 128
                    && (first.is_ascii_alphabetic() || first == b'_')
                    && rest.iter().all(|&b| is_word(b))
            }
            None => false,
        };
        if !valid {
            return Err(ValidationError::new(
                "variable name",
                "must be 1 to 128 letters, digits or underscores, and not start with a digit",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(VarName);

/// An image reference as the admin types it, `[registry[:port]/]path[:tag]`, by the grammar of
/// Docker's `distribution/reference`, without a digest. At most 255 characters.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageRef(String);

impl ImageRef {
    /// Checks the Docker reference grammar.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let (name, tag) = split_tag(raw);
        if raw.len() > MAX_IMAGE_REF_LEN || !is_image_name(name) || !tag.is_none_or(is_tag) {
            return Err(ValidationError::new(
                "image",
                "must be a Docker image reference of at most 255 characters, such as nginx:1.29 \
                 or ghcr.io/org/app:v1",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(ImageRef);

/// An image pinned by its digest, `<name>@sha256:<64 lowercase hex>`, as Docker reports it in
/// `RepoDigests`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageDigestRef(String);

impl ImageDigestRef {
    /// Checks the image name and the digest.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let valid = match raw.split_once("@sha256:") {
            Some((name, digest)) => {
                name.len() <= MAX_IMAGE_REF_LEN
                    && is_image_name(name)
                    && digest.len() == 64
                    && digest.bytes().all(is_lower_hex)
            }
            None => false,
        };
        if !valid {
            return Err(ValidationError::new(
                "image digest",
                "must be an image name followed by @sha256: and 64 lowercase hexadecimal \
                 characters",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(ImageDigestRef);

/// A command that replaces the image's: 1 to 1,024 characters, without NUL or newline. It is
/// split into argv as a POSIX shell would split it, but no shell runs it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StartCommand(String);

impl StartCommand {
    /// Checks the length and the forbidden characters, then that the command splits into at
    /// least one argument.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        if !(1..=1024).contains(&raw.chars().count()) || raw.contains(['\0', '\n']) {
            return Err(ValidationError::new(
                "start command",
                "must be 1 to 1,024 characters, without NUL or newline",
            ));
        }
        match shell_words::split(raw) {
            Ok(argv) if !argv.is_empty() => Ok(Self(raw.to_owned())),
            _ => Err(ValidationError::new(
                "start command",
                "must have at least one argument, and its quotes must be closed",
            )),
        }
    }

    /// The program, then its arguments.
    pub fn argv(&self) -> Vec<String> {
        shell_words::split(&self.0).expect("parse checked that the command splits")
    }
}

string_newtype!(StartCommand);

/// The path of an HTTP healthcheck: it starts with `/` and has at most 256 characters from
/// `[A-Za-z0-9/_.~!$&'()*+,;=:@%?-]`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpPath(String);

impl HttpPath {
    /// Checks the leading slash, the length and the characters.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let is_allowed = |b: u8| b.is_ascii_alphanumeric() || b"/_.~!$&'()*+,;=:@%?-".contains(&b);
        if !raw.starts_with('/') || raw.len() > 256 || !raw.bytes().all(is_allowed) {
            return Err(ValidationError::new(
                "healthcheck path",
                "must start with / and have at most 256 letters, digits or URL characters",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(HttpPath);

/// An absolute path inside a container, where a volume mounts: at most 256 characters from
/// `[A-Za-z0-9/_.-]`. Not `/` itself, and no segment is empty, `.` or `..`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerPath(String);

impl ContainerPath {
    /// Checks the leading slash, the length, the characters and the segments.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let is_allowed = |b: u8| b.is_ascii_alphanumeric() || b"/_.-".contains(&b);
        let valid = match raw.strip_prefix('/') {
            Some(relative) => {
                raw.len() <= 256
                    && raw.bytes().all(is_allowed)
                    && relative
                        .split('/')
                        .all(|segment| !matches!(segment, "" | "." | ".."))
            }
            None => false,
        };
        if !valid {
            return Err(ValidationError::new(
                "container path",
                "must be an absolute path below / of at most 256 letters, digits, /, _, . or -, \
                 without empty, . or .. segments",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(ContainerPath);

/// 1 to 64 characters, no control character.
fn parse_display_name(field: &'static str, raw: &str) -> Result<String, ValidationError> {
    if !(1..=64).contains(&raw.chars().count()) || raw.chars().any(char::is_control) {
        return Err(ValidationError::new(
            field,
            "must be 1 to 64 characters, without control characters",
        ));
    }
    Ok(raw.to_owned())
}

/// Splits `name[:tag]`: the tag follows the last colon, if no slash comes after it.
fn split_tag(reference: &str) -> (&str, Option<&str>) {
    match reference.rfind(':') {
        Some(colon) if !reference[colon..].contains('/') => {
            (&reference[..colon], Some(&reference[colon + 1..]))
        }
        _ => (reference, None),
    }
}

/// `[registry/]component(/component)*`. As in Docker, the first part is a registry when more
/// parts follow and it has a dot or a port, or is `localhost`.
fn is_image_name(name: &str) -> bool {
    let path = match name.split_once('/') {
        Some((registry, path)) if registry.contains(['.', ':']) || registry == "localhost" => {
            if !is_registry(registry) {
                return false;
            }
            path
        }
        _ => name,
    };
    path.split('/').all(is_path_component)
}

/// `host[:port]`, where the host is dot-separated DNS labels and the port 1 to 5 digits.
fn is_registry(registry: &str) -> bool {
    let (host, port) = match registry.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (registry, None),
    };
    let is_port =
        |port: &str| (1..=5).contains(&port.len()) && port.bytes().all(|b| b.is_ascii_digit());
    port.is_none_or(is_port) && host.split('.').all(is_host_label)
}

/// `[A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?`.
fn is_host_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    match (bytes.first(), bytes.last()) {
        (Some(first), Some(last)) => {
            first.is_ascii_alphanumeric()
                && last.is_ascii_alphanumeric()
                && bytes
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || *b == b'-')
        }
        _ => false,
    }
}

/// `[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*`.
fn is_path_component(component: &str) -> bool {
    let is_alphanumeric = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    let mut rest = component;
    loop {
        let run = rest.find(|c| !is_alphanumeric(c)).unwrap_or(rest.len());
        if run == 0 {
            return false;
        }
        rest = &rest[run..];
        if rest.is_empty() {
            return true;
        }
        let separator_len = rest.find(is_alphanumeric).unwrap_or(rest.len());
        let separator = &rest[..separator_len];
        if !matches!(separator, "." | "_" | "__") && !separator.bytes().all(|b| b == b'-') {
            return false;
        }
        rest = &rest[separator_len..];
    }
}

/// `[A-Za-z0-9_][A-Za-z0-9_.-]{0,127}`.
fn is_tag(tag: &str) -> bool {
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    match tag.as_bytes().split_first() {
        Some((&first, rest)) => {
            tag.len() <= 128
                && is_word(first)
                && rest.iter().all(|&b| is_word(b) || b == b'.' || b == b'-')
        }
        None => false,
    }
}

fn is_lower_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
}

#[cfg(test)]
mod tests {
    use super::{
        ContainerPath, HttpPath, ImageDigestRef, ImageRef, ProjectName, ServiceName, Slug,
        StartCommand, VarName,
    };

    const DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn project_name_has_1_to_64_characters_without_control() {
        let longest = "é".repeat(64);
        let too_long = "é".repeat(65);
        for valid in ["My shop 🚀", "a", longest.as_str()] {
            assert_eq!(
                ProjectName::parse(valid).map(|n| n.to_string()),
                Ok(valid.to_owned())
            );
        }
        for invalid in ["", too_long.as_str(), "a\nb", "a\tb", "a\u{7f}b"] {
            assert!(ProjectName::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn service_name_has_1_to_64_characters_without_control() {
        let longest = "x".repeat(64);
        let too_long = "x".repeat(65);
        for valid in ["API gateway", longest.as_str()] {
            assert_eq!(
                ServiceName::parse(valid).map(|n| n.to_string()),
                Ok(valid.to_owned())
            );
        }
        for invalid in ["", too_long.as_str(), "a\rb", "a\0b"] {
            assert!(ServiceName::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn slug_is_a_dns_label() {
        let longest = "a".repeat(63);
        let too_long = "a".repeat(64);
        for valid in ["api-prod", "a", "0", "a--b", "nginx-2", longest.as_str()] {
            assert_eq!(
                Slug::parse(valid).map(|s| s.as_str().to_owned()),
                Ok(valid.to_owned())
            );
        }
        for invalid in [
            "-api",
            "api-",
            "",
            "API",
            "a_b",
            "a.b",
            "é",
            too_long.as_str(),
        ] {
            assert!(Slug::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn var_name_is_an_identifier_of_at_most_128_characters() {
        let longest = "A".repeat(128);
        let too_long = "A".repeat(129);
        for valid in ["DATABASE_URL", "_x", "a1", longest.as_str()] {
            assert_eq!(
                VarName::parse(valid).map(|n| n.as_str().to_owned()),
                Ok(valid.to_owned())
            );
        }
        for invalid in ["1X", "", "A-B", "A B", "A=B", "É", too_long.as_str()] {
            assert!(VarName::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn image_ref_follows_the_docker_reference_grammar() {
        let longest = format!("{}:{}", "a".repeat(126), "t".repeat(128));
        let too_long = format!("{}:{}", "a".repeat(127), "t".repeat(128));
        let tag_too_long = format!("nginx:{}", "t".repeat(129));
        for valid in [
            "nginx:1.29",
            "ghcr.io/org/app:v1",
            "localhost:5000/x",
            "nginx",
            "library/nginx:1.29-alpine",
            "registry.example.com:443/team/my_app__x/web-ui:1.0.0-rc.1",
            "a.b--c/d:_",
            longest.as_str(),
        ] {
            assert_eq!(
                ImageRef::parse(valid).map(|i| i.as_str().to_owned()),
                Ok(valid.to_owned())
            );
        }
        for invalid in [
            "-x",
            "a b",
            "",
            "Nginx",
            "nginx:",
            "nginx:-x",
            "nginx:a/b",
            "ghcr.io/",
            "/nginx",
            "a//b",
            "a_",
            "a..b",
            "a___b",
            "-registry.io/x",
            "registry.io:port/x",
            "nginx\n",
            "nginx@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            tag_too_long.as_str(),
            too_long.as_str(),
        ] {
            assert!(ImageRef::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn image_digest_ref_is_a_name_and_a_sha256_digest() {
        for name in ["nginx", "ghcr.io/org/app", "localhost:5000/x"] {
            let valid = format!("{name}@{DIGEST}");
            assert_eq!(
                ImageDigestRef::parse(&valid).map(|i| i.as_str().to_owned()),
                Ok(valid)
            );
        }
        let uppercase = format!(
            "nginx@{}",
            DIGEST.to_uppercase().replace("SHA256", "sha256")
        );
        let short = format!("nginx@{}", &DIGEST[..DIGEST.len() - 1]);
        let tagged = format!("nginx:1.29@{DIGEST}");
        let other_algorithm = format!("nginx@{}", DIGEST.replace("sha256", "sha512"));
        let bad_name = format!("-x@{DIGEST}");
        for invalid in [
            "nginx",
            "nginx:1.29",
            DIGEST,
            uppercase.as_str(),
            short.as_str(),
            tagged.as_str(),
            other_algorithm.as_str(),
            bad_name.as_str(),
        ] {
            assert!(ImageDigestRef::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn start_command_splits_into_argv() {
        let command = StartCommand::parse("node server.js --port 3000").expect("valid command");
        assert_eq!(command.argv(), ["node", "server.js", "--port", "3000"]);
        assert_eq!(command.as_str(), "node server.js --port 3000");

        let quoted = StartCommand::parse(r#"sh -c 'echo "hi" $HOME'"#).expect("valid command");
        assert_eq!(quoted.argv(), ["sh", "-c", r#"echo "hi" $HOME"#]);

        let longest = "a".repeat(1024);
        assert!(StartCommand::parse(&longest).is_ok());
        let too_long = "a".repeat(1025);
        for invalid in [
            "",
            "   ",
            "''x'",
            "echo 'x",
            "a\nb",
            "a\0b",
            too_long.as_str(),
        ] {
            assert!(StartCommand::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn http_path_is_an_absolute_path_of_at_most_256_characters() {
        let longest = format!("/{}", "a".repeat(255));
        let too_long = format!("/{}", "a".repeat(256));
        for valid in [
            "/",
            "/health",
            "/api/v1/ready?full=1&x=%20",
            "/~a!$&'()*+,;=:@-_.",
            longest.as_str(),
        ] {
            assert_eq!(
                HttpPath::parse(valid).map(|p| p.as_str().to_owned()),
                Ok(valid.to_owned())
            );
        }
        for invalid in [
            "",
            "health",
            "/a b",
            "/é",
            "/a#b",
            "/a\"b",
            "/a\nb",
            too_long.as_str(),
        ] {
            assert!(HttpPath::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn container_path_is_absolute_and_never_climbs() {
        let longest = format!("/{}", "a".repeat(255));
        let too_long = format!("/{}", "a".repeat(256));
        for valid in [
            "/data",
            "/var/lib/postgresql/data",
            "/a.b/c-d_e",
            "/...",
            longest.as_str(),
        ] {
            assert_eq!(
                ContainerPath::parse(valid).map(|p| p.as_str().to_owned()),
                Ok(valid.to_owned())
            );
        }
        for invalid in [
            "",
            "data",
            "/data/../etc",
            "/..",
            "/../x",
            "/a b",
            "/a:b",
            "/é",
            too_long.as_str(),
        ] {
            assert!(ContainerPath::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn container_path_is_never_the_root() {
        assert!(ContainerPath::parse("/").is_err());
    }

    #[test]
    fn container_path_has_no_empty_segment() {
        for invalid in ["//", "//data", "/data//x", "/data/"] {
            assert!(ContainerPath::parse(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn container_path_has_no_dot_segment() {
        for invalid in ["/.", "/./data", "/data/.", "/data/./x"] {
            assert!(ContainerPath::parse(invalid).is_err(), "{invalid:?}");
        }
        assert!(ContainerPath::parse("/.data/x.").is_ok());
    }

    #[test]
    fn deserializing_validates() {
        assert_eq!(
            serde_json::from_str::<Slug>(r#""api-prod""#).ok(),
            Slug::parse("api-prod").ok()
        );
        assert!(serde_json::from_str::<Slug>(r#""-api""#).is_err());
        assert!(serde_json::from_str::<ImageRef>(r#""a b""#).is_err());
        let command = StartCommand::parse("node server.js").expect("valid command");
        assert_eq!(
            serde_json::to_string(&command).ok(),
            Some(r#""node server.js""#.to_owned())
        );
    }
}
