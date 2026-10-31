use std::sync::{Arc, Mutex};

use tokio::sync::{broadcast, watch};

use crate::config::LogLevel;

/// Where the SDK is in its lifecycle. `ShutDown` is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SdkState {
    /// Configured and ready to start; no background work runs yet.
    Initialized,
    /// Background work is running.
    Running,
    /// Stopped for good; every call that needs the engine fails with `NomiError::ShutDown`.
    ShutDown,
}

/// One diagnostic line from inside the SDK.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct LogRecord {
    pub level: LogLevel,
    /// The Rust module that logged it, e.g. `nomi_sdk::client`.
    pub target: String,
    pub message: String,
    pub timestamp_ms: u64,
}

/// Everything the SDK tells its subscribers.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum SdkEvent {
    StateChanged {
        state: SdkState,
    },
    /// The open account changed: `Some` after one was opened, `None` after it was closed or removed.
    AccountChanged {
        account_id: Option<String>,
    },
    Log {
        record: LogRecord,
    },
    /// The subscriber fell behind and `count` events were dropped before it caught up.
    EventsDropped {
        count: u64,
    },
}

/// Fans [`SdkEvent`]s out to every live [`EventSubscription`].
pub(crate) struct EventBus {
    sender: Mutex<Option<broadcast::Sender<SdkEvent>>>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Mutex::new(Some(sender)),
        }
    }

    /// Delivers `event` to the current subscribers; a closed bus or no subscribers drops it.
    pub fn emit(&self, event: SdkEvent) {
        if let Some(sender) = self.sender.lock().expect("event bus poisoned").as_ref() {
            let _ = sender.send(event);
        }
    }

    /// Subscribers drain what was already sent, then their stream ends.
    pub fn close(&self) {
        self.sender.lock().expect("event bus poisoned").take();
    }

    pub fn subscribe(&self) -> Arc<EventSubscription> {
        let receiver = self
            .sender
            .lock()
            .expect("event bus poisoned")
            .as_ref()
            .map(broadcast::Sender::subscribe);
        Arc::new(EventSubscription::new(receiver))
    }
}

/// A stream of [`SdkEvent`]s, pulled one at a time with [`EventSubscription::next_event`].
///
/// It sees events sent after it was created. It ends (`None`) after the SDK shuts down and the
/// remaining events are drained, or as soon as [`EventSubscription::cancel`] is called.
#[derive(uniffi::Object)]
pub struct EventSubscription {
    receiver: tokio::sync::Mutex<Option<broadcast::Receiver<SdkEvent>>>,
    closed: watch::Sender<bool>,
}

impl EventSubscription {
    fn new(receiver: Option<broadcast::Receiver<SdkEvent>>) -> Self {
        Self {
            receiver: tokio::sync::Mutex::new(receiver),
            closed: watch::Sender::new(false),
        }
    }
}

#[uniffi::export]
impl EventSubscription {
    /// The next event, or `None` once the stream has ended.
    ///
    /// Runtime-agnostic: it only waits on channels, so the host's executor may poll it.
    pub async fn next_event(&self) -> Option<SdkEvent> {
        let mut closed = self.closed.subscribe();
        if *closed.borrow() {
            return None;
        }
        let mut receiver = self.receiver.lock().await;
        let stream = receiver.as_mut()?;
        tokio::select! {
            biased;
            _ = closed.wait_for(|closed| *closed) => None,
            received = stream.recv() => match received {
                Ok(event) => Some(event),
                Err(broadcast::error::RecvError::Lagged(count)) => Some(SdkEvent::EventsDropped { count }),
                Err(broadcast::error::RecvError::Closed) => {
                    receiver.take();
                    None
                }
            },
        }
    }

    /// Ends the stream; a pending [`EventSubscription::next_event`] returns `None`.
    ///
    /// Named `cancel`, not `close`: Kotlin objects already have `close()` (`AutoCloseable`), which
    /// frees the native handle.
    pub fn cancel(&self) {
        self.closed.send_replace(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPACITY: usize = 16;

    fn running() -> SdkEvent {
        SdkEvent::StateChanged {
            state: SdkState::Running,
        }
    }

    #[tokio::test]
    async fn delivers_events_in_order_to_every_subscriber() {
        let bus = EventBus::new(CAPACITY);
        let (first, second) = (bus.subscribe(), bus.subscribe());
        bus.emit(running());
        bus.emit(SdkEvent::EventsDropped { count: 1 });
        for subscription in [first, second] {
            assert_eq!(subscription.next_event().await, Some(running()));
            assert_eq!(
                subscription.next_event().await,
                Some(SdkEvent::EventsDropped { count: 1 })
            );
        }
    }

    #[tokio::test]
    async fn drains_then_ends_after_the_bus_closes() {
        let bus = EventBus::new(CAPACITY);
        let subscription = bus.subscribe();
        bus.emit(running());
        bus.close();
        assert_eq!(subscription.next_event().await, Some(running()));
        assert_eq!(subscription.next_event().await, None);
        assert_eq!(subscription.next_event().await, None);
    }

    #[tokio::test]
    async fn a_subscription_to_a_closed_bus_ends_at_once() {
        let bus = EventBus::new(CAPACITY);
        bus.close();
        assert_eq!(bus.subscribe().next_event().await, None);
    }

    #[tokio::test]
    async fn cancel_wakes_a_pending_next_event() {
        let bus = EventBus::new(CAPACITY);
        let subscription = bus.subscribe();
        let pending = tokio::spawn({
            let subscription = subscription.clone();
            async move { subscription.next_event().await }
        });
        tokio::task::yield_now().await;
        subscription.cancel();
        assert_eq!(pending.await.unwrap(), None);
    }

    #[tokio::test]
    async fn a_slow_subscriber_is_told_how_many_events_it_missed() {
        let bus = EventBus::new(CAPACITY);
        let subscription = bus.subscribe();
        let overflow = 3;
        for _ in 0..CAPACITY + overflow {
            bus.emit(running());
        }
        assert_eq!(
            subscription.next_event().await,
            Some(SdkEvent::EventsDropped {
                count: overflow as u64
            })
        );
        assert_eq!(subscription.next_event().await, Some(running()));
    }
}
