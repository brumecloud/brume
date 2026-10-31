//! The engine service. It runs `brume-engine-core` over SQLite storage, listens on the API,
//! admin and agent sockets, relays log streams from telemetry and ships the event log to the
//! bucket.

#![forbid(unsafe_code)]
