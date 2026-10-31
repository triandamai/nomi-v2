use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::config::{SdkConfig, ValidatedConfig};
use crate::error::NomiError;
use crate::event::{EventBus, EventSubscription, SdkEvent, SdkState};
use crate::logging;

/// Worker threads for the SDK's runtime: background work is I/O-bound, and phones are small.
const WORKER_THREADS: usize = 2;
const THREAD_NAME: &str = "nomi-sdk";
/// How long [`NomiSdk::shutdown`] waits for background tasks before abandoning them.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// A snapshot of the SDK's health, for support screens and bug reports.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Diagnostics {
    pub sdk_version: String,
    pub state: SdkState,
    pub base_url: String,
    pub data_dir: String,
    /// Wall-clock milliseconds since the epoch when the SDK last started; `None` until then.
    pub started_at_ms: Option<u64>,
}

/// The SDK's entry point: one per signed-in app process, owned by the app, not by a screen.
///
/// `new` initializes (validates the config, prepares the data directory, builds the runtime),
/// `start` launches background work, `shutdown` stops it for good. All three are safe to call
/// from any thread; `start` and `shutdown` are idempotent.
#[derive(uniffi::Object)]
pub struct NomiSdk {
    config: ValidatedConfig,
    bus: Arc<EventBus>,
    inner: Mutex<Inner>,
}

struct Inner {
    state: SdkState,
    runtime: Option<Runtime>,
    stop: Option<watch::Sender<bool>>,
    tasks: Vec<JoinHandle<()>>,
    started_at_ms: Option<u64>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // A runtime dropped normally blocks, which panics inside another async context.
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

#[uniffi::export]
impl NomiSdk {
    #[uniffi::constructor]
    pub fn new(config: SdkConfig) -> Result<Arc<Self>, NomiError> {
        let config = config.validate()?;
        std::fs::create_dir_all(&config.data_dir).map_err(|error| NomiError::Storage {
            reason: format!(
                "can't create data_dir `{}`: {error}",
                config.data_dir.display()
            ),
        })?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(WORKER_THREADS)
            .thread_name(THREAD_NAME)
            .enable_all()
            .build()
            .map_err(|error| NomiError::Internal {
                reason: format!("can't build the async runtime: {error}"),
            })?;
        logging::init();

        let sdk = Arc::new(Self {
            bus: Arc::new(EventBus::new(config.event_buffer_capacity)),
            config,
            inner: Mutex::new(Inner {
                state: SdkState::Initialized,
                runtime: Some(runtime),
                stop: None,
                tasks: Vec::new(),
                started_at_ms: None,
            }),
        });
        tracing::info!(base_url = %sdk.config.base_url, "initialized");
        Ok(sdk)
    }

    /// Launches background work. A no-op while already running.
    pub async fn start(&self) -> Result<(), NomiError> {
        {
            let mut inner = self.lock();
            match inner.state {
                SdkState::Running => return Ok(()),
                SdkState::ShutDown => return Err(NomiError::ShutDown),
                SdkState::Initialized => {}
            }
            let runtime = inner.runtime.as_ref().ok_or(NomiError::ShutDown)?;
            let (stop, stopped) = watch::channel(false);
            let forwarder = runtime.spawn(logging::forward(
                logging::subscribe(),
                self.config.log_level,
                self.bus.clone(),
                stopped,
            ));
            inner.tasks.push(forwarder);
            inner.stop = Some(stop);
            inner.state = SdkState::Running;
            inner.started_at_ms = Some(logging::now_ms());
        }
        self.bus.emit(SdkEvent::StateChanged {
            state: SdkState::Running,
        });
        tracing::info!("started");
        Ok(())
    }

    /// Stops background work and ends every event stream. A no-op once shut down.
    pub async fn shutdown(&self) {
        let (runtime, stop, tasks) = {
            let mut inner = self.lock();
            if inner.state == SdkState::ShutDown {
                return;
            }
            inner.state = SdkState::ShutDown;
            (
                inner.runtime.take(),
                inner.stop.take(),
                std::mem::take(&mut inner.tasks),
            )
        };
        tracing::info!("shutting down");
        if let Some(stop) = stop {
            stop.send_replace(true);
        }
        if let Some(runtime) = runtime {
            let drained = runtime.spawn(async move {
                let joined = async {
                    for task in tasks {
                        let _ = task.await;
                    }
                };
                if tokio::time::timeout(SHUTDOWN_GRACE, joined).await.is_err() {
                    tracing::warn!("background tasks outlived the shutdown grace period");
                }
            });
            let _ = drained.await;
            runtime.shutdown_background();
        }
        self.bus.emit(SdkEvent::StateChanged {
            state: SdkState::ShutDown,
        });
        self.bus.close();
    }

    pub fn state(&self) -> SdkState {
        self.lock().state
    }

    /// A new stream of the events sent from now on.
    pub fn subscribe(&self) -> Arc<EventSubscription> {
        self.bus.subscribe()
    }

    pub fn diagnostics(&self) -> Diagnostics {
        let inner = self.lock();
        Diagnostics {
            sdk_version: crate::sdk_version(),
            state: inner.state,
            base_url: self.config.base_url.to_string(),
            data_dir: self.config.data_dir.display().to_string(),
            started_at_ms: inner.started_at_ms,
        }
    }
}

impl NomiSdk {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("SDK state poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LogLevel;
    use crate::default_sdk_config;

    const BASE_URL: &str = "https://api-nomi.trian.space";
    const EVENT_TIMEOUT: Duration = Duration::from_secs(5);

    fn sdk(dir: &tempfile::TempDir) -> Arc<NomiSdk> {
        NomiSdk::new(default_sdk_config(
            BASE_URL.to_owned(),
            dir.path().display().to_string(),
        ))
        .unwrap()
    }

    /// The next event that isn't a log line (other tests share the process-wide log channel).
    async fn next_lifecycle_event(subscription: &EventSubscription) -> Option<SdkEvent> {
        tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                match subscription.next_event().await {
                    Some(SdkEvent::Log { .. }) => continue,
                    other => return other,
                }
            }
        })
        .await
        .expect("timed out waiting for an event")
    }

    #[tokio::test]
    async fn walks_through_the_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        assert_eq!(sdk.state(), SdkState::Initialized);
        assert_eq!(sdk.diagnostics().started_at_ms, None);

        sdk.start().await.unwrap();
        assert_eq!(sdk.state(), SdkState::Running);
        assert!(sdk.diagnostics().started_at_ms.is_some());

        sdk.shutdown().await;
        assert_eq!(sdk.state(), SdkState::ShutDown);
    }

    #[tokio::test]
    async fn start_and_shutdown_are_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        sdk.start().await.unwrap();
        sdk.start().await.unwrap();
        sdk.shutdown().await;
        sdk.shutdown().await;
        assert_eq!(sdk.state(), SdkState::ShutDown);
    }

    #[tokio::test]
    async fn start_after_shutdown_fails() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        sdk.shutdown().await;
        assert_eq!(sdk.start().await, Err(NomiError::ShutDown));
    }

    #[tokio::test]
    async fn subscribers_see_state_changes_then_the_stream_ends() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        let subscription = sdk.subscribe();
        sdk.start().await.unwrap();
        sdk.shutdown().await;

        assert_eq!(
            next_lifecycle_event(&subscription).await,
            Some(SdkEvent::StateChanged {
                state: SdkState::Running
            })
        );
        assert_eq!(
            next_lifecycle_event(&subscription).await,
            Some(SdkEvent::StateChanged {
                state: SdkState::ShutDown
            })
        );
        assert_eq!(next_lifecycle_event(&subscription).await, None);
        assert_eq!(sdk.subscribe().next_event().await, None);
    }

    #[tokio::test]
    async fn forwards_sdk_logs_at_the_configured_level() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = default_sdk_config(BASE_URL.to_owned(), dir.path().display().to_string());
        config.log_level = LogLevel::Info;
        let sdk = NomiSdk::new(config).unwrap();
        let subscription = sdk.subscribe();
        sdk.start().await.unwrap();

        let marker = format!("marker-{}", logging::now_ms());
        tracing::debug!(target: "nomi_sdk::test", "{marker} hidden");
        tracing::info!(target: "nomi_sdk::test", "{marker} shown");

        let record = tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                if let Some(SdkEvent::Log { record }) = subscription.next_event().await {
                    if record.message.starts_with(&marker) {
                        return record;
                    }
                }
            }
        })
        .await
        .expect("the info line never arrived");
        assert_eq!(record.level, LogLevel::Info);
        assert_eq!(record.message, format!("{marker} shown"));
        sdk.shutdown().await;
    }

    #[tokio::test]
    async fn creates_the_data_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("accounts").join("default");
        NomiSdk::new(default_sdk_config(
            BASE_URL.to_owned(),
            nested.display().to_string(),
        ))
        .unwrap();
        assert!(nested.is_dir());
    }

    #[test]
    fn reports_a_data_dir_it_cannot_create() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("occupied");
        std::fs::write(&file, b"").unwrap();
        let config = default_sdk_config(
            BASE_URL.to_owned(),
            file.join("nested").display().to_string(),
        );
        assert!(matches!(
            NomiSdk::new(config),
            Err(NomiError::Storage { .. })
        ));
    }

    #[tokio::test]
    async fn dropping_a_running_sdk_inside_a_runtime_does_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        sdk.start().await.unwrap();
        drop(sdk);
    }

    #[test]
    fn can_be_driven_without_a_host_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        let host = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        host.block_on(sdk.start()).unwrap();
        host.block_on(sdk.shutdown());
        assert_eq!(sdk.state(), SdkState::ShutDown);
    }
}
