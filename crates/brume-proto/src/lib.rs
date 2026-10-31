//! Messages between Brume's services, and the DTOs of the HTTP API. Transport types only:
//! fields are strings and integers, which the receiving core parses into the domain's newtypes.
//! Every socket carries the frames of `frame`.

#![forbid(unsafe_code)]

pub mod admin;
pub mod agent;
pub mod api;
pub mod frame;
mod redact;
pub mod telemetry;
