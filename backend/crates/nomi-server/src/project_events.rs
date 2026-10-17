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
