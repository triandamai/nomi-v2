//! Applying realtime events to storage: one lane per topic, each applied strictly in order.
//!
//! Deltas must append in order, and a finished turn must not clear a draft before the deltas
//! queued ahead of it are written; but a fetch in one chat shouldn't hold up another.

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::mpsc;

use super::{Shared, USER_TOPIC};
use crate::error::NomiError;
use crate::event::{EventSubscription, SdkEvent};
use crate::network::events::{chat_of, RealtimeEvent};
use crate::network::realtime::ConnectionState;
use crate::storage::models::{AgentActivity, NotificationItem};
use crate::storage::repositories::{activity, drafts, notifications, sessions};

/// Finished turns remembered per chat, to drop their late deltas; turns finish one at a time.
const FINISHED_TURNS_KEPT: usize = 8;
/// `AgentSessionStarted` carries no phase; a freshly spawned agent is working.
const STARTED_PHASE: &str = "thinking";
/// The agent type of an agent first seen through a phase change.
const UNKNOWN_AGENT_TYPE: &str = "unknown";
const ROLE_ASSISTANT: &str = "assistant";

#[derive(Debug)]
pub(crate) enum Work {
    Event(RealtimeEvent),
    /// State may have been missed: refetch, and drop what only live events would have ended.
    Resync,
    /// Refetch.
    Refresh,
}

/// Routes the SDK's events to the lanes, and the connection coming back to the outbox.
pub(super) async fn dispatch(shared: Arc<Shared>, events: Arc<EventSubscription>) {
    while let Some(event) = events.next_event().await {
        match event {
            SdkEvent::Realtime { topic, event, .. } => shared.route(&topic, Work::Event(event)),
            SdkEvent::TopicResynced { topic } => shared.route(&topic, Work::Resync),
            // Events were lost: whatever they would have changed must be refetched.
            SdkEvent::EventsDropped { .. } => shared.resync_all(),
            SdkEvent::ConnectionChanged {
                state: ConnectionState::Connected { .. },
            } => {
                if let Err(error) = shared.retry_now().await {
                    tracing::warn!(%error, "sync: can't bring the outbox forward");
                }
            }
            SdkEvent::SignedIn { .. } => shared.wake.notify_one(),
            _ => {}
        }
    }
}

impl Shared {
    /// Queues `work` on `topic`'s lane, starting the lane if needed. Topics sync doesn't keep
    /// (projects) are ignored.
    pub(super) fn route(self: &Arc<Self>, topic: &str, work: Work) {
        if topic != USER_TOPIC && chat_of(topic).is_none() {
            return;
        }
        if self.stopped.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let mut lanes = self.lanes.lock().expect("lanes poisoned");
        let sender = lanes.entry(topic.to_owned()).or_insert_with(|| {
            let (sender, receiver) = mpsc::unbounded_channel();
            let task = self
                .runtime
                .spawn(run_lane(self.clone(), topic.to_owned(), receiver));
            self.lane_tasks.lock().expect("lanes poisoned").push(task);
            sender
        });
        let _ = sender.send(work);
    }

    pub(super) fn resync_all(self: &Arc<Self>) {
        self.route(USER_TOPIC, Work::Resync);
        for session_id in self.bound_chats() {
            self.route(&super::chat_lane(&session_id), Work::Resync);
        }
    }

    async fn apply(
        &self,
        topic: &str,
        work: Work,
        finished: &mut VecDeque<String>,
    ) -> Result<(), NomiError> {
        match chat_of(topic) {
            Some(session_id) => self.apply_chat(session_id, work, finished).await,
            None => self.apply_user(work).await,
        }
    }

    async fn apply_chat(
        &self,
        session_id: &str,
        work: Work,
        finished: &mut VecDeque<String>,
    ) -> Result<(), NomiError> {
        let session = session_id.to_owned();
        let now = Self::now();
        match work {
            Work::Refresh => self.refresh_messages(session_id).await,
            Work::Resync => {
                // A turn that is still live recreates its draft and agents with its next event.
                let refreshed = self.refresh_messages(session_id).await;
                self.store
                    .write(move |tx| {
                        drafts::clear(tx, &session, None)?;
                        activity::clear_for_session(tx, &session)
                    })
                    .await?;
                refreshed
            }
            Work::Event(event) => match event {
                RealtimeEvent::ChatDelta {
                    turn_job_id,
                    text: Some(text),
                    ..
                } => {
                    // A delta delivered after its turn ended must not resurrect the draft.
                    if finished.contains(&turn_job_id) {
                        return Ok(());
                    }
                    self.store
                        .write(move |tx| drafts::append(tx, &session, &turn_job_id, &text, now))
                        .await
                }
                RealtimeEvent::TurnCompleted {
                    turn_job_id,
                    message_id,
                    ..
                } => {
                    finish(finished, &turn_job_id);
                    let fetched = self.fetch_message(session_id, &message_id).await;
                    // Cleared even if the fetch failed (the next refresh brings the message), but
                    // only this turn's draft: a newer turn may already be streaming.
                    self.store
                        .write(move |tx| drafts::clear(tx, &session, Some(&turn_job_id)))
                        .await?;
                    fetched.map(drop)
                }
                RealtimeEvent::TurnFailed {
                    turn_job_id, error, ..
                } => {
                    finish(finished, &turn_job_id);
                    let turn = turn_job_id.clone();
                    self.store
                        .write(move |tx| drafts::clear(tx, &session, Some(&turn)))
                        .await?;
                    self.bus.emit(SdkEvent::ChatTurnFailed {
                        session_id: session_id.to_owned(),
                        turn_job_id,
                        error,
                    });
                    Ok(())
                }
                RealtimeEvent::MessageCreated { message_id, .. } => {
                    // The person's own message (say, from another device) must not wipe a reply
                    // that is still streaming.
                    let created = self.fetch_message(session_id, &message_id).await?;
                    if created.is_some_and(|message| message.role == ROLE_ASSISTANT) {
                        self.store
                            .write(move |tx| drafts::clear(tx, &session, None))
                            .await?;
                    }
                    Ok(())
                }
                RealtimeEvent::MessageUpdated { message_id, .. } => {
                    self.fetch_message(session_id, &message_id).await.map(drop)
                }
                RealtimeEvent::SessionRenamed { title, .. } => {
                    self.store
                        .write(move |tx| sessions::rename(tx, &session, &title))
                        .await
                }
                RealtimeEvent::AgentSessionStarted {
                    agent_session_id,
                    agent_type,
                    ..
                } => {
                    let row = AgentActivity {
                        agent_session_id,
                        session_id: session,
                        agent_type,
                        phase: STARTED_PHASE.to_owned(),
                        detail: None,
                        updated_at: now,
                    };
                    self.store.write(move |tx| activity::upsert(tx, &row)).await
                }
                RealtimeEvent::AgentPhaseChanged {
                    agent_session_id,
                    phase,
                    detail,
                    ..
                } => {
                    self.store
                        .write(move |tx| {
                            let known = activity::get(tx.raw(), &agent_session_id)?;
                            let row = AgentActivity {
                                // Started before this device was listening: the type is unknown
                                // until it starts again.
                                agent_type: known
                                    .map(|row| row.agent_type)
                                    .unwrap_or_else(|| UNKNOWN_AGENT_TYPE.to_owned()),
                                agent_session_id,
                                session_id: session,
                                phase,
                                detail,
                                updated_at: now,
                            };
                            activity::upsert(tx, &row)
                        })
                        .await
                }
                RealtimeEvent::AgentSessionEnded {
                    agent_session_id, ..
                } => {
                    self.store
                        .write(move |tx| activity::delete(tx, &agent_session_id))
                        .await
                }
                _ => Ok(()),
            },
        }
    }

    async fn apply_user(&self, work: Work) -> Result<(), NomiError> {
        let now = Self::now();
        match work {
            Work::Resync | Work::Refresh => {
                // One failed refetch must not skip the other.
                let sessions = self.refresh_sessions().await;
                let inbox = self.refresh_notifications().await;
                sessions.and(inbox)
            }
            Work::Event(event) => match event {
                RealtimeEvent::NotificationCreated {
                    notification_id,
                    category,
                    title,
                    body,
                    link,
                } => {
                    self.store
                        .write(move |tx| {
                            let id = notification_id.to_lowercase();
                            // The event has no timestamp: arrival time stands in until a refresh.
                            let created_at = notifications::get(tx.raw(), &id)?
                                .map_or(now, |known| known.created_at);
                            let item = NotificationItem {
                                id,
                                kind: category,
                                title,
                                body,
                                link,
                                created_at,
                                read_at: None,
                            };
                            notifications::upsert(tx, &[item])
                        })
                        .await
                }
                RealtimeEvent::ChatReply {
                    session_id,
                    preview,
                    ..
                } => {
                    let unread = !self.is_foreground(&session_id);
                    self.store
                        .write(move |tx| {
                            sessions::touch(tx, &session_id, Some(&preview), now, Some(unread))
                        })
                        .await
                }
                // The crew stopped in that chat: its agents aren't working any more.
                RealtimeEvent::ChatActivity { session_id, .. } => {
                    self.store
                        .write(move |tx| activity::clear_for_session(tx, &session_id))
                        .await
                }
                RealtimeEvent::ChatUpdated {
                    session_id, title, ..
                } => {
                    let known = self
                        .store
                        .write(move |tx| {
                            if sessions::get(tx.raw(), &session_id)?.is_none() {
                                return Ok(false);
                            }
                            sessions::touch(tx, &session_id, None, now, None)?;
                            if let Some(title) = title.filter(|title| !title.is_empty()) {
                                sessions::rename(tx, &session_id, &title)?;
                            }
                            Ok(true)
                        })
                        .await?;
                    // A chat this device hasn't seen: the list has it.
                    if known {
                        Ok(())
                    } else {
                        self.refresh_sessions().await
                    }
                }
                _ => Ok(()),
            },
        }
    }
}

async fn run_lane(shared: Arc<Shared>, topic: String, mut work: mpsc::UnboundedReceiver<Work>) {
    let mut finished = VecDeque::new();
    while let Some(item) = work.recv().await {
        match shared.apply(&topic, item, &mut finished).await {
            Ok(()) => {}
            Err(NomiError::AccountClosed | NomiError::ShutDown) => return,
            Err(error) => tracing::warn!(%topic, %error, "sync: applying a change failed"),
        }
    }
}

fn finish(finished: &mut VecDeque<String>, turn_job_id: &str) {
    if finished.iter().any(|turn| turn == turn_job_id) {
        return;
    }
    finished.push_back(turn_job_id.to_owned());
    if finished.len() > FINISHED_TURNS_KEPT {
        finished.pop_front();
    }
}
