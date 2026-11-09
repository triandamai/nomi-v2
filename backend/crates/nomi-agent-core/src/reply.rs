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
pub async fn within_reply<F: Future>(work: F) -> F::Output {
    if current_reply().is_some() {
        work.await
    } else {
        REPLY_ID.scope(Uuid::new_v4(), work).await
    }
}

/// Runs `work` as part of `reply` — a turn resumed after an approval carries on the reply it
/// paused in. A new reply when there is none to carry on.
pub async fn continuing_reply<F: Future>(reply: Option<Uuid>, work: F) -> F::Output {
    match reply {
        Some(id) => REPLY_ID.scope(id, work).await,
        None => within_reply(work).await,
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

    #[tokio::test]
    async fn a_resumed_turn_carries_on_the_reply_it_paused_in() {
        let paused = within_reply(async { current_reply() }).await;
        let resumed = continuing_reply(paused, within_reply(async { current_reply() })).await;
        assert_eq!(paused, resumed);
        assert!(continuing_reply(None, async { current_reply() }).await.is_some());
    }
}
