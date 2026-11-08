//! Live updates for a project's open pages: when something about its publishing changes (a
//! Shipyard event arrived, its address moved), the page hears it on `project/{id}/stream` and
//! reloads what it shows, instead of asking every few seconds.

use std::sync::OnceLock;

use nomi_realtime::MqttPublisher;
use serde_json::json;
use uuid::Uuid;

static PUBLISHER: OnceLock<MqttPublisher> = OnceLock::new();

/// Connects the publisher; until this runs (in tests, say) nothing is sent.
pub fn init(broker_host: &str, broker_port: u16) {
    let _ = PUBLISHER.get_or_init(|| MqttPublisher::connect(broker_host, broker_port, &format!("nomi-project-events-{}", Uuid::new_v4())));
}

/// The shared publisher, once connected: also for posting into a project's chat.
pub fn publisher() -> Option<&'static MqttPublisher> {
    PUBLISHER.get()
}

pub fn topic(project_id: Uuid) -> String {
    format!("project/{project_id}/stream")
}

/// Tells the project's open pages that its publishing changed. `what` is the Shipyard event
/// type (`publish.succeeded`, `domain.attached`, ...) or Nomi's own reason.
pub fn changed(project_id: Uuid, what: &str) {
    let Some(publisher) = PUBLISHER.get() else { return };
    if let Err(e) = publisher.try_publish_to(&topic(project_id), &json!({ "kind": "DeployChanged", "event": what })) {
        tracing::debug!(error = %e, %project_id, "couldn't tell the project's pages about a change");
    }
}

/// What the page hears when the sandbox moves to a new phase. It carries the phase only: the
/// preview address is fetched from the API by someone allowed to see it.
pub fn sandbox_changed_envelope(phase: &str) -> serde_json::Value {
    json!({ "kind": "SandboxChanged", "phase": phase })
}

type SandboxObserver = Box<dyn Fn(Uuid, &str) + Send + Sync>;

static SANDBOX_OBSERVER: OnceLock<SandboxObserver> = OnceLock::new();

/// Has `observer` called with every sandbox change, at the moment it is announced. For tests
/// that check what is saved by then; set once per process.
#[doc(hidden)]
pub fn observe_sandbox_changes_for_test(observer: impl Fn(Uuid, &str) + Send + Sync + 'static) {
    let _ = SANDBOX_OBSERVER.set(Box::new(observer));
}

/// Tells the project's open pages the sandbox moved to `phase`.
pub fn sandbox_changed(project_id: Uuid, phase: &str) {
    if let Some(observer) = SANDBOX_OBSERVER.get() {
        observer(project_id, phase);
    }
    let Some(publisher) = PUBLISHER.get() else { return };
    if let Err(e) = publisher.try_publish_to(&topic(project_id), &sandbox_changed_envelope(phase)) {
        tracing::debug!(error = %e, %project_id, "couldn't tell the project's pages the sandbox changed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_changed_carries_the_phase_and_nothing_else() {
        assert_eq!(sandbox_changed_envelope("running"), json!({ "kind": "SandboxChanged", "phase": "running" }));
    }
}
