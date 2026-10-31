//! When the agent may start a stopped container again. Docker's restart policy is off: the agent
//! restarts crashed containers itself, after 1 s, then 2 s, 4 s… up to 60 s.

use std::collections::BTreeMap;

use crate::observed::{DockerSnapshot, SnapState};

/// The wait after the first start.
pub const FIRST_BACKOFF_MS: u64 = 1_000;

/// The longest wait between two starts. A container that runs this long has its wait reset.
pub const MAX_BACKOFF_MS: u64 = 60_000;

/// The starts of each container, kept by the agent across reconciliations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Restarts {
    entries: BTreeMap<String, Starts>,
}

/// The starts of one container.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Starts {
    /// Every start, the first one included.
    total: u32,
    /// The starts since the last reset, which set the wait.
    attempts: u32,
    /// The earliest instant of the next start.
    next_allowed_ms: u64,
}

impl Restarts {
    /// Records that the agent started `name` at `now_ms`, and doubles its wait.
    pub fn record_start(&mut self, name: &str, now_ms: u64) {
        let starts = self.entries.entry(name.to_owned()).or_default();
        starts.total = starts.total.saturating_add(1);
        starts.attempts = starts.attempts.saturating_add(1);
        starts.next_allowed_ms = now_ms.saturating_add(wait_ms(starts.attempts));
    }

    /// Clears the wait of `name`. Its restart count stays.
    pub fn reset(&mut self, name: &str) {
        if let Some(starts) = self.entries.get_mut(name) {
            starts.attempts = 0;
            starts.next_allowed_ms = 0;
        }
    }

    /// Whether `name` may start at `now_ms`.
    pub fn allows_start(&self, name: &str, now_ms: u64) -> bool {
        self.entries
            .get(name)
            .is_none_or(|starts| now_ms >= starts.next_allowed_ms)
    }

    /// How many times the agent started `name` again after its first start.
    pub fn restarts(&self, name: &str) -> u32 {
        self.entries
            .get(name)
            .map_or(0, |starts| starts.total.saturating_sub(1))
    }

    /// Forgets the containers that are gone, and resets the wait of those that have run for
    /// `MAX_BACKOFF_MS` since their last start.
    pub fn settle(&mut self, snapshot: &DockerSnapshot, now_ms: u64) {
        self.entries
            .retain(|name, _| snapshot.container(name).is_some());
        for container in &snapshot.containers {
            let stable = container.state == SnapState::Running
                && container
                    .started_at_ms
                    .is_some_and(|started| now_ms.saturating_sub(started) >= MAX_BACKOFF_MS);
            if stable {
                self.reset(&container.name);
            }
        }
    }
}

/// The wait after the `attempts`-th start: 1 s, doubled each time, at most `MAX_BACKOFF_MS`.
fn wait_ms(attempts: u32) -> u64 {
    // Past 2^6 seconds the wait is capped anyway; the bound keeps the shift from overflowing.
    let doublings = attempts.saturating_sub(1).min(16);
    (FIRST_BACKOFF_MS << doublings).min(MAX_BACKOFF_MS)
}
