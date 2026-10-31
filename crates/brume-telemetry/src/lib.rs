//! The telemetry service. It stores container logs in a hardened DuckDB and serves them,
//! history then live, to the engine.

#![forbid(unsafe_code)]
