//! The agent's decisions. It compares the desired state with a Docker snapshot, plans the
//! Docker actions, builds container configs by allowlist and reports container health. No I/O.

#![forbid(unsafe_code)]

pub mod backoff;
pub mod config;
pub mod health;
pub mod observed;
pub mod plan;
