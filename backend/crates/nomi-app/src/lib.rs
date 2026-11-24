//! The binaries: `nomi-orchestrator` (the HTTP server, running the worker loops too unless told
//! not to) and `worker` (the loops on their own).

/// Whether the server runs the worker loops itself, from `RUN_WORKER_INLINE`: yes unless it is
/// "false" or "0", so one process is enough for a single instance.
pub fn worker_inline(setting: Option<&str>) -> bool {
    setting.map_or(true, |v| v != "false" && v != "0")
}
