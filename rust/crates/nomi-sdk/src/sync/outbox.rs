//! The outbox worker: delivers queued writes in order per lane, retries with jittered backoff, and
//! replays whatever a previous run left behind.
//!
//! The backend has no idempotency keys, so a message whose attempt may have reached it (a timeout,
//! a dropped connection, a crash mid-request) is never simply posted again: the worker first looks
//! for it in the chat's newest page, and claims a match instead of sending a duplicate.

use std::sync::Arc;
use std::time::Duration;

use super::{Shared, MESSAGE_PAGE_SIZE};
use crate::error::NomiError;
use crate::network::realtime::backoff_delay;
use crate::storage::models::{Message, OutboxOperation, STATUS_FAILED};
use crate::storage::repositories::{messages, outbox as queue};

const ROLE_USER: &str = "user";

/// What a failed attempt means for its entry.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// Worth trying again later: offline, a timeout, the backend overloaded or failing.
    Retry,
    /// Not the entry's fault: there is no usable session right now. Wait without counting it.
    Wait,
    /// The account closed or the SDK stopped; leave everything as it is.
    Stop,
    /// The backend refused it; trying again won't help.
    Refused,
}

const HTTP_REQUEST_TIMEOUT: u16 = 408;
const HTTP_TOO_MANY_REQUESTS: u16 = 429;
const HTTP_SERVER_ERRORS: u16 = 500;

fn verdict(error: &NomiError) -> Verdict {
    match error {
        NomiError::Network { .. } | NomiError::InvalidResponse { .. } => Verdict::Retry,
        NomiError::Http { status, .. }
            if *status >= HTTP_SERVER_ERRORS
                || *status == HTTP_REQUEST_TIMEOUT
                || *status == HTTP_TOO_MANY_REQUESTS =>
        {
            Verdict::Retry
        }
        NomiError::Unauthorized | NomiError::NotSignedIn => Verdict::Wait,
        NomiError::AccountClosed | NomiError::ShutDown => Verdict::Stop,
        _ => Verdict::Refused,
    }
}

/// Whether the backend may have acted on a request that ended in `error`: it answered nothing
/// usable, so it can't be ruled out.
fn may_have_landed(error: &NomiError) -> bool {
    matches!(
        error,
        NomiError::Network { .. } | NomiError::InvalidResponse { .. }
    )
}

pub(super) async fn run(shared: Arc<Shared>) {
    let mut session = shared.api.auth().watch_session();
    shared.prune().await;
    loop {
        if !shared.is_owner() {
            tokio::select! {
                changed = session.changed() => if changed.is_err() { return },
                _ = shared.wake.notified() => {}
            }
            continue;
        }
        let now = Shared::now();
        match shared
            .store
            .read(move |connection| queue::next_due(connection, now))
            .await
        {
            Ok(Some(item)) => shared.attempt(item).await,
            Ok(None) => {
                let wake_at = match shared.store.read(queue::next_wake).await {
                    Ok(wake_at) => wake_at,
                    Err(NomiError::AccountClosed | NomiError::ShutDown) => return,
                    Err(error) => {
                        tracing::warn!(%error, "sync: can't read the outbox");
                        Some(now + millis(shared.settings.retry_base))
                    }
                };
                let sleep = wake_at.map(|at| Duration::from_millis((at - now).max(0) as u64));
                tokio::select! {
                    _ = shared.wake.notified() => {}
                    changed = session.changed() => if changed.is_err() { return },
                    _ = async { tokio::time::sleep(sleep.unwrap_or_default()).await }, if sleep.is_some() => {}
                }
            }
            Err(NomiError::AccountClosed | NomiError::ShutDown) => return,
            Err(error) => {
                tracing::warn!(%error, "sync: can't read the outbox");
                tokio::time::sleep(shared.settings.retry_base).await;
            }
        }
    }
}

fn millis(duration: Duration) -> i64 {
    duration.as_millis() as i64
}

impl Shared {
    /// Forgets idempotency records past their retention.
    async fn prune(&self) {
        let before = Self::now() - millis(self.settings.done_retention);
        match self.store.write(move |tx| queue::prune(tx, before)).await {
            Ok(0) => {}
            Ok(pruned) => tracing::debug!(pruned, "sync: pruned finished outbox entries"),
            Err(error) => tracing::warn!(%error, "sync: can't prune the outbox"),
        }
    }

    async fn attempt(&self, item: queue::Item) {
        match item.operation.clone() {
            OutboxOperation::SendMessage {
                session_id,
                text,
                local_message_id,
            } => {
                self.deliver_message(item, &session_id, &text, &local_message_id)
                    .await
            }
            OutboxOperation::MarkNotificationRead { notification_id } => {
                let api = self.api.clone();
                self.deliver(item, async move {
                    api.mark_notification_read(&notification_id).await
                })
                .await
            }
            OutboxOperation::MarkAllNotificationsRead => {
                let api = self.api.clone();
                self.deliver(item, async move { api.mark_all_notifications_read().await })
                    .await
            }
        }
    }

    /// Delivers an idempotent write (marking read): safe to repeat, so no echo check.
    async fn deliver(
        &self,
        item: queue::Item,
        request: impl std::future::Future<Output = Result<(), NomiError>>,
    ) {
        if !self.begin(&item).await {
            return;
        }
        let outcome = request.await;
        let (seq, now) = (item.seq, Self::now());
        let settled = match outcome {
            Ok(()) => {
                self.store
                    .write(move |tx| queue::complete(tx, seq, None, now))
                    .await
            }
            Err(error) if verdict(&error) == Verdict::Refused => {
                // A read marker the backend won't take isn't worth keeping around.
                tracing::warn!(%error, key = %item.key, "sync: the backend refused a read marker; dropped");
                self.store
                    .write(move |tx| queue::complete(tx, seq, None, now))
                    .await
            }
            Err(error) => return self.settle_failure(&item, error, false).await,
        };
        log_settle(settled);
    }

    async fn deliver_message(
        &self,
        item: queue::Item,
        session_id: &str,
        text: &str,
        local_message_id: &str,
    ) {
        if item.in_flight {
            match self.find_echo(&item, session_id, text).await {
                Ok(Some(found)) => {
                    tracing::info!(key = %item.key, message_id = %found.id, "sync: the backend already had a queued message");
                    return self
                        .finish_message(&item, local_message_id, vec![found])
                        .await;
                }
                Ok(None) => {}
                Err(error) => return self.settle_failure(&item, error, true).await,
            }
        }
        if !self.begin(&item).await {
            return;
        }
        let sent = self
            .api
            .send_message(session_id, text)
            .await
            .and_then(|sent| {
                let mut stored = vec![sent.user_message.into_message(session_id)?];
                if let Some(reply) = sent.supervisor_reply {
                    stored.push(reply.into_message(session_id)?);
                }
                Ok(stored)
            });
        match sent {
            Ok(stored) => self.finish_message(&item, local_message_id, stored).await,
            Err(error) => {
                let landed = may_have_landed(&error);
                self.settle_failure(&item, error, landed).await
            }
        }
    }

    /// The server's copy of `item`'s message, if an earlier attempt landed: the person's message
    /// with the same text, created no earlier than the entry (less the clock-skew window), and not
    /// already claimed by an earlier entry of the chat.
    async fn find_echo(
        &self,
        item: &queue::Item,
        session_id: &str,
        text: &str,
    ) -> Result<Option<Message>, NomiError> {
        let page = self
            .api
            .messages(session_id, None, MESSAGE_PAGE_SIZE)
            .await?;
        let lane = item.lane.clone();
        let claimed = self
            .store
            .read(move |connection| queue::results_in_lane(connection, &lane))
            .await?;
        let since = item.created_at - millis(self.settings.echo_window);
        for dto in page.messages {
            if dto.sender != ROLE_USER || dto.content != text {
                continue;
            }
            let message = dto.into_message(session_id)?;
            if message.created_at >= since && !claimed.contains(&message.id) {
                return Ok(Some(message));
            }
        }
        Ok(None)
    }

    /// The backend has the message: its copy replaces the local one, and the entry is done.
    async fn finish_message(
        &self,
        item: &queue::Item,
        local_message_id: &str,
        stored: Vec<Message>,
    ) {
        let (seq, now, local) = (item.seq, Self::now(), local_message_id.to_owned());
        let settled = self
            .store
            .write(move |tx| {
                messages::delete(tx, &local)?;
                messages::upsert(tx, &stored)?;
                let result_id = stored.first().map(|message| message.id.as_str());
                queue::complete(tx, seq, result_id, now)
            })
            .await;
        log_settle(settled);
        self.prune().await;
    }

    /// Records that an attempt is going out. False when the entry can't be touched any more.
    async fn begin(&self, item: &queue::Item) -> bool {
        let (seq, now) = (item.seq, Self::now());
        match self
            .store
            .write(move |tx| queue::begin_attempt(tx, seq, now))
            .await
        {
            Ok(()) => true,
            Err(error) => {
                if !matches!(error, NomiError::AccountClosed | NomiError::ShutDown) {
                    tracing::warn!(%error, "sync: can't record an outbox attempt");
                }
                false
            }
        }
    }

    /// Reschedules or fails `item` after `error`. `in_flight`: the backend may have it.
    async fn settle_failure(&self, item: &queue::Item, error: NomiError, in_flight: bool) {
        let (seq, now, reason) = (item.seq, Self::now(), error.to_string());
        let attempts = item.attempts + 1;
        let settings = &self.settings;
        let give_up = |reason: String| {
            let local = match &item.operation {
                OutboxOperation::SendMessage {
                    local_message_id, ..
                } => Some(local_message_id.clone()),
                _ => None,
            };
            move |tx: &mut crate::storage::Tx<'_>| {
                queue::fail(tx, seq, &reason, now)?;
                if let Some(local) = &local {
                    messages::set_status(tx, local, STATUS_FAILED)?;
                }
                Ok(())
            }
        };
        let verdict = match verdict(&error) {
            // Refused although the session is still there: the backend's problem, so it counts.
            Verdict::Wait if self.is_owner() => Verdict::Retry,
            other => other,
        };
        let settled = match verdict {
            Verdict::Stop => return,
            Verdict::Refused => {
                tracing::warn!(%error, key = %item.key, "sync: the backend refused an outbox entry");
                self.store.write(give_up(reason)).await
            }
            Verdict::Retry if attempts >= settings.max_attempts => {
                tracing::warn!(%error, key = %item.key, attempts, "sync: an outbox entry ran out of retries");
                self.store.write(give_up(reason)).await
            }
            Verdict::Retry => {
                let delay = backoff_delay(attempts, settings.retry_base, settings.retry_max);
                let next = now + millis(delay);
                tracing::info!(%error, key = %item.key, attempts, "sync: an outbox entry will be retried");
                self.store
                    .write(move |tx| {
                        queue::reschedule(tx, seq, attempts, next, in_flight, &reason, now)
                    })
                    .await
            }
            Verdict::Wait => {
                // Not counted against the entry: it goes again once there is a session.
                let (previous, next) = (item.attempts, now + millis(settings.retry_base));
                self.store
                    .write(move |tx| {
                        queue::reschedule(tx, seq, previous, next, in_flight, &reason, now)
                    })
                    .await
            }
        };
        log_settle(settled);
    }
}

fn log_settle(settled: Result<(), NomiError>) {
    match settled {
        Ok(()) | Err(NomiError::AccountClosed | NomiError::ShutDown) => {}
        Err(error) => tracing::warn!(%error, "sync: can't record an outbox result"),
    }
}
