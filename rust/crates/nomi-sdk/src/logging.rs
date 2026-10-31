//! `tracing` inside the SDK, surfaced to the host as [`SdkEvent::Log`].
//!
//! One process-wide subscriber feeds a broadcast channel; each running [`crate::NomiSdk`] forwards
//! the records its configured level allows onto its own event bus. Only the SDK's own targets are
//! captured, so dependency chatter never crosses the FFI boundary.

use std::fmt::{self, Write as _};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{broadcast, watch};
use tracing::field::{Field, Visit};
use tracing::subscriber::Interest;
use tracing::{Event, Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt as _};

use crate::config::LogLevel;
use crate::event::{EventBus, LogRecord, SdkEvent};

/// Records buffered for forwarders that fall behind.
const LOG_CHANNEL_CAPACITY: usize = 1_024;
const SDK_TARGET: &str = env!("CARGO_CRATE_NAME");
const MESSAGE_FIELD: &str = "message";

static LOG_CHANNEL: OnceLock<broadcast::Sender<LogRecord>> = OnceLock::new();

/// Installs the process-wide subscriber on first use. If the host process already installed a
/// global `tracing` subscriber, that one wins and the SDK forwards no logs.
pub(crate) fn init() {
    log_channel();
}

fn log_channel() -> &'static broadcast::Sender<LogRecord> {
    LOG_CHANNEL.get_or_init(|| {
        let (sender, _) = broadcast::channel(LOG_CHANNEL_CAPACITY);
        let subscriber = tracing_subscriber::registry().with(ForwardLayer {
            sender: sender.clone(),
        });
        let _ = tracing::subscriber::set_global_default(subscriber);
        sender
    })
}

/// The records logged from now on, for [`forward`]. Taken before spawning the forwarder so lines
/// logged while it is being scheduled aren't lost.
pub(crate) fn subscribe() -> broadcast::Receiver<LogRecord> {
    log_channel().subscribe()
}

/// Forwards `records` allowed by `level` onto `bus` until `stop` turns true.
pub(crate) async fn forward(
    mut records: broadcast::Receiver<LogRecord>,
    level: LogLevel,
    bus: Arc<EventBus>,
    mut stop: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            biased;
            _ = stop.wait_for(|stop| *stop) => break,
            received = records.recv() => match received {
                Ok(record) if level.allows(record.level) => bus.emit(SdkEvent::Log { record }),
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(count)) => bus.emit(SdkEvent::EventsDropped { count }),
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
}

struct ForwardLayer {
    sender: broadcast::Sender<LogRecord>,
}

impl ForwardLayer {
    fn is_sdk_target(target: &str) -> bool {
        target == SDK_TARGET
            || target
                .strip_prefix(SDK_TARGET)
                .is_some_and(|rest| rest.starts_with("::"))
    }
}

impl<S: Subscriber> Layer<S> for ForwardLayer {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if Self::is_sdk_target(metadata.target()) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
        Self::is_sdk_target(metadata.target())
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let metadata = event.metadata();
        let mut message = MessageVisitor::default();
        event.record(&mut message);
        let _ = self.sender.send(LogRecord {
            level: metadata.level().into(),
            target: metadata.target().to_owned(),
            message: message.finish(),
            timestamp_ms: now_ms(),
        });
    }
}

/// `message` first, then every other field as ` key=value`.
#[derive(Default)]
struct MessageVisitor {
    message: String,
    fields: String,
}

impl MessageVisitor {
    fn finish(self) -> String {
        self.message + &self.fields
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == MESSAGE_FIELD {
            let _ = write!(self.message, "{value:?}");
        } else {
            let _ = write!(self.fields, " {}={value:?}", field.name());
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == MESSAGE_FIELD {
            self.message.push_str(value);
        } else {
            let _ = write!(self.fields, " {}={value}", field.name());
        }
    }
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_only_the_sdks_own_targets() {
        assert!(ForwardLayer::is_sdk_target("nomi_sdk"));
        assert!(ForwardLayer::is_sdk_target("nomi_sdk::client"));
        assert!(!ForwardLayer::is_sdk_target("nomi_sdk_extra"));
        assert!(!ForwardLayer::is_sdk_target("hyper::client"));
    }
}
