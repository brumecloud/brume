//! Identifiers. Entities get UUIDv7s, which sort by creation time; blobs get 128 random bits.

use uuid::{Builder, Uuid, Variant};

use crate::ValidationError;
use crate::ctx::Ctx;

/// Declares an entity identifier: a UUIDv7 in canonical form, drawn from a `Ctx`.
macro_rules! uuid_id {
    ($(#[$attr:meta])* $name:ident, $field:literal) => {
        $(#[$attr])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Draws a new identifier from the context's clock and randomness.
            pub fn new(ctx: &mut impl Ctx) -> Self {
                Self(new_uuid_v7(ctx))
            }

            /// Parses a UUIDv7 in canonical form: 36 lowercase characters with hyphens.
            pub fn parse(raw: &str) -> Result<Self, ValidationError> {
                parse_uuid_v7($field, raw).map(Self)
            }
        }

        string_newtype!($name);
    };
}

uuid_id!(
    /// A project.
    ProjectId,
    "project id"
);
uuid_id!(
    /// An environment of a project.
    EnvironmentId,
    "environment id"
);
uuid_id!(
    /// A service of an environment.
    ServiceId,
    "service id"
);
uuid_id!(
    /// A deployment of a service.
    DeploymentId,
    "deployment id"
);
uuid_id!(
    /// A revision: the immutable spec that a deployment puts in service.
    RevisionId,
    "revision id"
);
uuid_id!(
    /// A variable or a secret, at the project, environment or service level.
    VariableId,
    "variable id"
);
uuid_id!(
    /// A staged change, waiting for the deploy of its environment.
    ChangeId,
    "change id"
);
uuid_id!(
    /// An administrator.
    AdminId,
    "admin id"
);
uuid_id!(
    /// A machine that runs an agent.
    MachineId,
    "machine id"
);

/// A blob of the engine's store: 128 random bits, as 32 lowercase hexadecimal characters.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobId(String);

impl BlobId {
    /// Draws a new identifier from the context's randomness.
    pub fn new(ctx: &mut impl Ctx) -> Self {
        let mut bytes = [0; 16];
        ctx.fill_random(&mut bytes);
        Self(hex::encode(bytes))
    }

    /// Parses 32 lowercase hexadecimal characters.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        let is_lower_hex = |b: u8| b.is_ascii_digit() || (b'a'..=b'f').contains(&b);
        if raw.len() != 32 || !raw.bytes().all(is_lower_hex) {
            return Err(ValidationError::new(
                "blob id",
                "must be 32 lowercase hexadecimal characters",
            ));
        }
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(BlobId);

fn new_uuid_v7(ctx: &mut impl Ctx) -> String {
    let mut random = [0; 10];
    ctx.fill_random(&mut random);
    Builder::from_unix_timestamp_millis(ctx.now().as_millis(), &random)
        .into_uuid()
        .hyphenated()
        .to_string()
}

fn parse_uuid_v7(field: &'static str, raw: &str) -> Result<String, ValidationError> {
    let invalid = || ValidationError::new(field, "must be a UUIDv7 in lowercase, with hyphens");
    let uuid = Uuid::try_parse(raw).map_err(|_| invalid())?;
    let canonical = uuid.hyphenated().to_string();
    if uuid.get_version_num() != 7 || uuid.get_variant() != Variant::RFC4122 || canonical != raw {
        return Err(invalid());
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::{AdminId, BlobId, ProjectId};
    use crate::ctx::Ctx;
    use crate::time::Timestamp;

    struct TestCtx {
        now: Timestamp,
        rng: ChaCha8Rng,
    }

    impl TestCtx {
        fn new() -> Self {
            Self {
                now: Timestamp::from_millis(1_793_450_096_789),
                rng: ChaCha8Rng::seed_from_u64(7),
            }
        }
    }

    impl Ctx for TestCtx {
        fn now(&self) -> Timestamp {
            self.now
        }

        fn fill_random(&mut self, buf: &mut [u8]) {
            rand::Rng::fill_bytes(&mut self.rng, buf);
        }
    }

    #[test]
    fn id_is_a_canonical_uuid_v7_from_the_context() {
        let mut ctx = TestCtx::new();
        let first = ProjectId::new(&mut ctx);
        let second = ProjectId::new(&mut ctx);
        assert_ne!(first, second);

        let raw = first.as_str();
        assert_eq!(raw.len(), 36);
        assert_eq!(&raw[14..15], "7");
        // The first 48 bits are the context's time in milliseconds.
        let millis = u64::from_str_radix(&raw[..13].replace('-', ""), 16).expect("hex time");
        assert_eq!(millis, 1_793_450_096_789);

        assert_eq!(ProjectId::parse(raw), Ok(first.clone()));
        assert_eq!(first.to_string(), raw);
        assert_eq!(
            serde_json::to_string(&first).ok(),
            Some(format!("\"{raw}\""))
        );
        assert_eq!(
            serde_json::from_str::<ProjectId>(&format!("\"{raw}\"")).ok(),
            Some(first)
        );
    }

    #[test]
    fn id_parse_accepts_only_canonical_uuid_v7() {
        assert!(AdminId::parse("0190a3f2-7c1e-7b3a-9f00-1234567890ab").is_ok());
        for invalid in [
            "",
            "not-a-uuid",
            // Version 4.
            "0190a3f2-7c1e-4b3a-9f00-1234567890ab",
            // Not the RFC 9562 variant.
            "0190a3f2-7c1e-7b3a-1f00-1234567890ab",
            "0190A3F2-7C1E-7B3A-9F00-1234567890AB",
            "0190a3f27c1e7b3a9f001234567890ab",
            "{0190a3f2-7c1e-7b3a-9f00-1234567890ab}",
            "urn:uuid:0190a3f2-7c1e-7b3a-9f00-1234567890ab",
        ] {
            assert!(AdminId::parse(invalid).is_err(), "{invalid:?}");
        }
        assert!(serde_json::from_str::<AdminId>(r#""not-a-uuid""#).is_err());
    }

    #[test]
    fn blob_id_is_128_random_bits_in_lowercase_hex() {
        let mut ctx = TestCtx::new();
        let blob = BlobId::new(&mut ctx);
        assert_eq!(blob.as_str().len(), 32);
        assert!(
            blob.as_str()
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
        assert_ne!(blob, BlobId::new(&mut ctx));
        assert_eq!(BlobId::parse(blob.as_str()), Ok(blob.clone()));

        let valid = "0123456789abcdef0123456789abcdef";
        assert!(BlobId::parse(valid).is_ok());
        for invalid in [
            "",
            &valid[1..],
            "0123456789ABCDEF0123456789ABCDEF",
            "0123456789abcdef0123456789abcdeg",
        ] {
            assert!(BlobId::parse(invalid).is_err(), "{invalid:?}");
        }
        let too_long = format!("{valid}0");
        assert!(BlobId::parse(&too_long).is_err());
    }
}
