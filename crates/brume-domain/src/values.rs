//! Numeric settings and the values of variables and secrets.

use secrecy::{ExposeSecret, SecretString};

use crate::ValidationError;

/// The largest value of a variable or a secret, in bytes.
pub const MAX_VALUE_BYTES: usize = 32 * 1024;

/// Declares an integer newtype bounded by `MIN..=MAX`.
macro_rules! bounded_integer {
    ($(#[$attr:meta])* $name:ident($int:ty), $min:literal..=$max:literal, $field:literal, $message:literal) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name($int);

        impl $name {
            /// The smallest valid value.
            pub const MIN: $int = $min;
            /// The largest valid value.
            pub const MAX: $int = $max;

            /// Checks the bounds.
            pub fn new(value: $int) -> Result<Self, ValidationError> {
                if !(Self::MIN..=Self::MAX).contains(&value) {
                    return Err(ValidationError::new($field, $message));
                }
                Ok(Self(value))
            }

            /// The validated value.
            pub const fn get(self) -> $int {
                self.0
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <$int as serde::Deserialize>::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

bounded_integer!(
    /// A TCP port that a service listens on: 1 to 65535.
    Port(u16),
    1..=65535,
    "port",
    "must be between 1 and 65535"
);

bounded_integer!(
    /// A CPU limit in thousandths of a CPU: 100 to 8000.
    CpuMillis(u32),
    100..=8000,
    "CPU limit",
    "must be between 0.1 and 8 CPUs"
);

bounded_integer!(
    /// A memory limit in mebibytes: 64 to 16384.
    MemoryMb(u32),
    64..=16384,
    "memory limit",
    "must be between 64 MiB and 16 GiB"
);

/// The value of a plain or build variable: at most 32 KiB, without NUL, kept byte for byte.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlainValue(String);

impl PlainValue {
    /// Checks the size and the absence of NUL.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        check_value("value", raw)?;
        Ok(Self(raw.to_owned()))
    }
}

string_newtype!(PlainValue);

/// The value of a secret. Same rules as `PlainValue`, but held in a `SecretString`: `Debug`
/// masks it, and it has neither `Display` nor `Serialize`.
#[derive(Clone, Debug)]
pub struct SecretValue(SecretString);

impl SecretValue {
    /// Checks the size and the absence of NUL.
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        check_value("secret", raw)?;
        Ok(Self(SecretString::from(raw)))
    }

    /// The secret, still wrapped.
    pub fn into_secret_string(self) -> SecretString {
        self.0
    }
}

impl ExposeSecret<str> for SecretValue {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

fn check_value(field: &'static str, raw: &str) -> Result<(), ValidationError> {
    if raw.len() > MAX_VALUE_BYTES || raw.contains('\0') {
        return Err(ValidationError::new(
            field,
            "must be at most 32 KiB, without NUL",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::{CpuMillis, MAX_VALUE_BYTES, MemoryMb, PlainValue, Port, SecretValue};

    #[test]
    fn port_is_between_1_and_65535() {
        assert!(Port::new(0).is_err());
        assert_eq!(Port::new(1).map(Port::get), Ok(1));
        assert_eq!(Port::new(65535).map(Port::get), Ok(65535));
        assert!(serde_json::from_str::<Port>("0").is_err());
        assert_eq!(
            serde_json::from_str::<Port>("8080").ok(),
            Port::new(8080).ok()
        );
    }

    #[test]
    fn cpu_millis_is_between_100_and_8000() {
        assert!(CpuMillis::new(99).is_err());
        assert_eq!(CpuMillis::new(100).map(CpuMillis::get), Ok(100));
        assert_eq!(CpuMillis::new(8000).map(CpuMillis::get), Ok(8000));
        assert!(CpuMillis::new(8001).is_err());
    }

    #[test]
    fn memory_mb_is_between_64_and_16384() {
        assert!(MemoryMb::new(63).is_err());
        assert_eq!(MemoryMb::new(64).map(MemoryMb::get), Ok(64));
        assert_eq!(MemoryMb::new(16384).map(MemoryMb::get), Ok(16384));
        assert!(MemoryMb::new(16385).is_err());
    }

    #[test]
    fn plain_value_keeps_bytes_up_to_32_kib_without_nul() {
        assert_eq!(MAX_VALUE_BYTES, 32 * 1024);
        let special = "a\nb=\"c\" 🚀";
        assert_eq!(
            PlainValue::parse(special).map(|v| v.as_str().to_owned()),
            Ok(special.into())
        );
        assert_eq!(
            PlainValue::parse("").map(|v| v.as_str().to_owned()),
            Ok(String::new())
        );

        let largest = "x".repeat(32_768);
        assert_eq!(
            PlainValue::parse(&largest).map(|v| v.as_str().len()),
            Ok(32_768)
        );
        // 32 767 bytes and a 4-byte emoji: the limit counts bytes, not characters.
        let too_large = format!("{}🚀", "x".repeat(32_765));
        assert_eq!(too_large.len(), 32_769);
        assert!(PlainValue::parse(&too_large).is_err());
        assert!(PlainValue::parse("a\0b").is_err());
    }

    #[test]
    fn secret_value_has_plain_rules_and_a_masked_debug() {
        let secret = SecretValue::parse("s3cr3t-🚀").expect("valid secret");
        assert_eq!(secret.expose_secret(), "s3cr3t-🚀");
        assert!(!format!("{secret:?}").contains("s3cr3t"));

        assert!(SecretValue::parse(&"x".repeat(32_768)).is_ok());
        assert!(SecretValue::parse(&"x".repeat(32_769)).is_err());
        assert!(SecretValue::parse("a\0b").is_err());
    }
}
