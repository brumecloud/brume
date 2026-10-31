//! The level of a log line, guessed from its words: containers write plain text, without a
//! level field.

use brume_proto::api::LogLevel;
use brume_proto::telemetry::StdStream;

/// Words that make a line an error.
const ERROR_WORDS: [&str; 4] = ["error", "fatal", "panic", "exception"];

/// Words that make a line a warning.
const WARN_WORDS: [&str; 2] = ["warn", "warning"];

/// `Error` if `message` holds one of `ERROR_WORDS` as a whole word, in any case; else `Warn` for
/// one of `WARN_WORDS`; else `Info`. A word is a run of letters and digits: `terrorist` holds no
/// error. The stream does not count, because many apps write everything to stderr.
pub fn classify(_stream: StdStream, message: &str) -> LogLevel {
    let mut level = LogLevel::Info;
    for word in message.split(|c: char| !c.is_alphanumeric()) {
        if ERROR_WORDS.iter().any(|w| word.eq_ignore_ascii_case(w)) {
            return LogLevel::Error;
        }
        if WARN_WORDS.iter().any(|w| word.eq_ignore_ascii_case(w)) {
            level = LogLevel::Warn;
        }
    }
    level
}
