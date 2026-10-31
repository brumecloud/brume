//! The agent service, the only one that talks to Docker. It applies the engine's desired
//! state through bollard, resolves images and forwards container logs to telemetry.

#![forbid(unsafe_code)]
