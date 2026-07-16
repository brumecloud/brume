pub const RENDERER_WORKER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/brume-renderer"));
pub const WEB_RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.js"));
pub const WEB_THEME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/theme.css"));
