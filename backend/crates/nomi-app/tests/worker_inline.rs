//! RUN_WORKER_INLINE: the server runs the loops itself unless it is "false" or "0".

use nomi_app::worker_inline;

#[test]
fn inline_off_starts_no_loops() {
    assert!(!worker_inline(Some("false")));
    assert!(!worker_inline(Some("0")));
    assert!(worker_inline(None));
    assert!(worker_inline(Some("true")));
}
