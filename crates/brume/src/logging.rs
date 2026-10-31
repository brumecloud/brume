//! The `tracing` subscriber of each service: to stderr, each line prefixed by the service's name,
//! filtered by `RUST_LOG` (`info` by default).
//!
//! bollard logs the bodies of its requests and responses at `debug`, and a container's
//! environment holds its secrets: `bollard`, `hyper` and `h2` stay at `info` whatever `RUST_LOG`
//! says.

use std::fmt;
use std::io::IsTerminal as _;

use tracing::{Event, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// The level when `RUST_LOG` is unset or does not parse.
const DEFAULT_DIRECTIVE: &str = "info";

/// The directives added after `RUST_LOG`'s, which they override: these crates log request and
/// response bodies below `info`.
const FORCED_DIRECTIVES: [&str; 3] = ["bollard=info", "hyper=info", "h2=info"];

/// The width of the name column, the longest name and a space.
const NAME_WIDTH: usize = "telemetry".len();

/// Installs the subscriber of the service `name`. A second call does nothing.
pub fn init(name: &'static str) {
    let rust_log = std::env::var("RUST_LOG").ok();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter(rust_log.as_deref()))
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .event_format(Prefixed {
            name,
            inner: tracing_subscriber::fmt::format(),
        })
        .try_init();
}

/// `rust_log`'s directives, or `info`, then the forced ones.
pub fn filter(rust_log: Option<&str>) -> EnvFilter {
    let base = rust_log
        .filter(|value| !value.trim().is_empty())
        .and_then(|value| EnvFilter::try_new(value).ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_DIRECTIVE));
    FORCED_DIRECTIVES.iter().fold(base, |filter, directive| {
        filter.add_directive(
            directive
                .parse::<Directive>()
                .expect("a forced directive parses"),
        )
    })
}

/// A format that writes the service's name before each event.
struct Prefixed<F> {
    name: &'static str,
    inner: F,
}

impl<S, N, F> FormatEvent<S, N> for Prefixed<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    F: FormatEvent<S, N>,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        write!(writer, "{:<NAME_WIDTH$} ", self.name)?;
        self.inner.format_event(ctx, writer, event)
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::layer::SubscriberExt as _;

    use super::filter;

    /// Whether the filter built from `rust_log` lets through an event of `target` at `level`.
    fn enabled(rust_log: Option<&str>, target: &'static str, level: Level) -> bool {
        let subscriber = tracing_subscriber::registry().with(filter(rust_log));
        tracing::subscriber::with_default(subscriber, || match (target, level) {
            ("bollard::docker", Level::DEBUG) => {
                tracing::enabled!(target: "bollard::docker", Level::DEBUG)
            }
            ("bollard::docker", Level::INFO) => {
                tracing::enabled!(target: "bollard::docker", Level::INFO)
            }
            ("hyper::proto", Level::TRACE) => {
                tracing::enabled!(target: "hyper::proto", Level::TRACE)
            }
            ("h2::codec", Level::DEBUG) => tracing::enabled!(target: "h2::codec", Level::DEBUG),
            ("brume_agent", Level::DEBUG) => {
                tracing::enabled!(target: "brume_agent", Level::DEBUG)
            }
            ("brume_agent", Level::INFO) => tracing::enabled!(target: "brume_agent", Level::INFO),
            _ => unreachable!("no callsite for {target} at {level}"),
        })
    }

    #[test]
    fn bollard_debug_stays_off_whatever_rust_log_says() {
        for rust_log in [
            Some("debug"),
            Some("trace"),
            Some("bollard=trace"),
            Some("debug,bollard=debug"),
        ] {
            assert!(
                !enabled(rust_log, "bollard::docker", Level::DEBUG),
                "{rust_log:?}"
            );
            assert!(
                !enabled(rust_log, "hyper::proto", Level::TRACE),
                "{rust_log:?}"
            );
            assert!(
                !enabled(rust_log, "h2::codec", Level::DEBUG),
                "{rust_log:?}"
            );
            assert!(
                enabled(rust_log, "bollard::docker", Level::INFO),
                "{rust_log:?}"
            );
        }
        assert!(enabled(Some("debug"), "brume_agent", Level::DEBUG));
    }

    #[test]
    fn info_without_a_usable_rust_log() {
        for rust_log in [None, Some(""), Some("=not a directive=")] {
            assert!(
                enabled(rust_log, "brume_agent", Level::INFO),
                "{rust_log:?}"
            );
            assert!(
                !enabled(rust_log, "brume_agent", Level::DEBUG),
                "{rust_log:?}"
            );
        }
    }
}
