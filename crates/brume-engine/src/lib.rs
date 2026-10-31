//! The engine service. It runs `brume-engine-core` over SQLite storage, listens on the API,
//! admin and agent sockets, relays log streams from telemetry and ships the event log to the
//! bucket.

#![forbid(unsafe_code)]

mod relay;
mod shell;
mod shipper;
mod sqlite;

pub use shell::run;
pub use shipper::{
    GENERATION, MAX_SEGMENT_EVENTS, SHIP_PERIOD, ShipError, ShipOutcome, Shipper, segment_bytes,
    segment_key,
};
pub use sqlite::SqliteStorage;
