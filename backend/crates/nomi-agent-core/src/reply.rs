//! Which crew reply a tool call belongs to. Every agent turn (`run_agent_turn`, and the tools
//! it resumes after an approval in `resolve_tool_batch`) runs inside one reply, so the tools can
//! group what they change by it — Koda's file tools count one project version per reply, not
//! one per file.

use std::future::Future;

use uuid::Uuid;

tokio::task_local! {
    static REPLY_ID: Uuid;
}

/// The reply the current tool call is part of, or None outside an agent turn (the code editor,
/// an import).
pub fn current_reply() -> Option<Uuid> {
    REPLY_ID.try_with(|id| *id).ok()
}

/// Runs `work` as one reply. Inside a reply already (a crew member answering within another's
/// turn), it stays part of that one.
///
/// `work` is boxed: an agent turn's future is large, and holding it inline in this one (and in
/// the resume that wraps a turn) overflows the stack in debug builds.
pub fn within_reply<F: Future>(work: F) -> impl Future<Output = F::Output> {
    // Boxed here, not inside an async block, which would hold `work` inline until first polled.
    let work = Box::pin(work);
    async move {
        if current_reply().is_some() {
            work.await
        } else {
            REPLY_ID.scope(Uuid::new_v4(), work).await
        }
    }
}

/// Runs `work` as part of `reply` — a turn resumed after an approval carries on the reply it
/// paused in. A new reply when there is none to carry on.
/// Boxed like `within_reply`.
pub fn continuing_reply<F: Future>(reply: Option<Uuid>, work: F) -> impl Future<Output = F::Output> {
    let work = Box::pin(work);
    async move {
        match reply {
            Some(id) => REPLY_ID.scope(id, work).await,
            None => within_reply(work).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn there_is_no_reply_outside_a_turn() {
        assert_eq!(current_reply(), None);
    }

    #[tokio::test]
    async fn one_turn_keeps_one_reply_and_the_next_gets_a_new_one() {
        let (first, again) = within_reply(async { (current_reply(), current_reply()) }).await;
        assert!(first.is_some());
        assert_eq!(first, again);
        let second = within_reply(async { current_reply() }).await;
        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn a_nested_turn_stays_part_of_the_outer_reply() {
        let (outer, inner) = within_reply(async { (current_reply(), within_reply(async { current_reply() }).await) }).await;
        assert_eq!(outer, inner);
    }

    /// A future holding this much stays on the heap inside the reply wrappers.
    const LARGE_FUTURE_BYTES: usize = 64 * 1024;
    /// What the wrappers' own futures may weigh: a box and the task-local, not the work.
    const WRAPPER_MAX_BYTES: usize = 1024;

    #[test]
    fn the_wrappers_keep_a_large_turn_off_the_stack() {
        let large = || async {
            let buffer = [0u8; LARGE_FUTURE_BYTES];
            tokio::task::yield_now().await;
            buffer.len()
        };
        assert!(std::mem::size_of_val(&large()) >= LARGE_FUTURE_BYTES);
        assert!(std::mem::size_of_val(&within_reply(large())) < WRAPPER_MAX_BYTES);
        assert!(std::mem::size_of_val(&continuing_reply(Some(Uuid::new_v4()), large())) < WRAPPER_MAX_BYTES);
    }

    #[tokio::test]
    async fn a_resumed_turn_carries_on_the_reply_it_paused_in() {
        let paused = within_reply(async { current_reply() }).await;
        let resumed = continuing_reply(paused, within_reply(async { current_reply() })).await;
        assert_eq!(paused, resumed);
        assert!(continuing_reply(None, async { current_reply() }).await.is_some());
    }
}
