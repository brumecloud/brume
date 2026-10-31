//! Instants, as the engine records them.

use time::OffsetDateTime;
use time::format_description::BorrowedFormatItem;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

use crate::ValidationError;

/// RFC 3339 in UTC, to the millisecond.
const RFC3339_MILLIS: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

/// The last millisecond that RFC 3339 can write: `9999-12-31T23:59:59.999Z`.
const MAX_RFC3339_MILLIS: u64 = 253_402_300_799_999;

/// An instant, in milliseconds since the Unix epoch. It serializes in RFC 3339, in UTC, to the
/// millisecond: `"2026-10-31T12:34:56.789Z"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    /// The instant `millis` milliseconds after the Unix epoch.
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Milliseconds since the Unix epoch.
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    /// The instant `millis` milliseconds later, saturating at the largest instant.
    pub const fn plus_millis(self, millis: u64) -> Self {
        Self(self.0.saturating_add(millis))
    }

    /// RFC 3339 in UTC, to the millisecond. An instant past the year 9999 writes as its last
    /// millisecond.
    pub fn to_rfc3339(self) -> String {
        let nanos = i128::from(self.0.min(MAX_RFC3339_MILLIS)) * 1_000_000;
        OffsetDateTime::from_unix_timestamp_nanos(nanos)
            .expect("an instant up to the year 9999 is in range")
            .format(RFC3339_MILLIS)
            .expect("a UTC instant up to the year 9999 formats")
    }

    /// Parses an RFC 3339 instant with any offset, from 1970 on, to the millisecond at most.
    pub fn parse_rfc3339(raw: &str) -> Result<Self, ValidationError> {
        let invalid = || {
            ValidationError::new(
                "timestamp",
                "must be an RFC 3339 instant from 1970 on, to the millisecond at most",
            )
        };
        let nanos = OffsetDateTime::parse(raw, &Rfc3339)
            .map_err(|_| invalid())?
            .unix_timestamp_nanos();
        if nanos % 1_000_000 != 0 {
            return Err(invalid());
        }
        u64::try_from(nanos / 1_000_000)
            .map(Self)
            .map_err(|_| invalid())
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_rfc3339())
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse_rfc3339(&raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn formats_rfc3339_utc_with_milliseconds() {
        assert_eq!(
            Timestamp::from_millis(0).to_rfc3339(),
            "1970-01-01T00:00:00.000Z"
        );
        assert_eq!(
            Timestamp::from_millis(1_793_450_096_789).to_rfc3339(),
            "2026-10-31T12:34:56.789Z"
        );
        assert_eq!(
            Timestamp::from_millis(u64::MAX).to_rfc3339(),
            "9999-12-31T23:59:59.999Z"
        );
    }

    #[test]
    fn plus_millis_adds_and_saturates() {
        let start = Timestamp::from_millis(1_000);
        assert_eq!(start.plus_millis(300_000).as_millis(), 301_000);
        assert_eq!(
            Timestamp::from_millis(u64::MAX).plus_millis(1).as_millis(),
            u64::MAX
        );
        assert!(start < start.plus_millis(1));
    }

    #[test]
    fn serializes_as_rfc3339() {
        let at = Timestamp::from_millis(1_793_450_096_789);
        let json = serde_json::to_string(&at).expect("serializes");
        assert_eq!(json, r#""2026-10-31T12:34:56.789Z""#);
        assert_eq!(serde_json::from_str::<Timestamp>(&json).ok(), Some(at));
    }

    #[test]
    fn parse_rfc3339_converts_offsets_and_rejects_the_rest() {
        assert_eq!(
            Timestamp::parse_rfc3339("2026-10-31T14:34:56.789+02:00").map(Timestamp::as_millis),
            Ok(1_793_450_096_789)
        );
        assert_eq!(
            Timestamp::parse_rfc3339("2026-10-31T12:34:56Z").map(Timestamp::as_millis),
            Ok(1_793_450_096_000)
        );
        for invalid in [
            "",
            "2026-10-31",
            "1969-12-31T23:59:59.999Z",
            "2026-10-31T12:34:56.7891Z",
        ] {
            assert!(Timestamp::parse_rfc3339(invalid).is_err(), "{invalid:?}");
        }
    }
}
