//! The engine's state machine. It replays the event log into a `World`, turns API
//! calls and agent reports into commands, and returns what the shell must do. No I/O: the
//! shell supplies time, randomness and storage.

#![forbid(unsafe_code)]
