//! The Nomi client engine.
//!
//! Kotlin (Android, iOS, desktop) reaches this crate only through the types exported here with
//! UniFFI: [`NomiSdk`] for the lifecycle, [`EventSubscription`] for the typed event stream,
//! [`AccountStore`] for an account's local data, [`SdkConfig`] / [`SdkEvent`] / [`NomiError`] and
//! the storage models as the stable public types.

uniffi::setup_scaffolding!();

mod client;
mod config;
mod error;
mod event;
mod logging;
mod network;
mod services;
mod storage;
mod sync;
#[cfg(test)]
mod test_support;

pub use client::{Diagnostics, NomiSdk};
pub use config::{LogLevel, SdkConfig, DEFAULT_EVENT_BUFFER_CAPACITY};
pub use error::{EmailCodeError, NomiError};
pub use event::{EventSubscription, LogRecord, SdkEvent, SdkState};
pub use network::auth::{
    CredentialStore, Credentials, Profile, SignInResult, SignOutReason, SignedInUser,
    VerificationInfo,
};
pub use network::events::RealtimeEvent;
pub use network::realtime::{ConnectionState, TopicWatch};
pub use services::account::{
    ConnectionsOverview, GoogleConnection, HealthConnection, Memory, Project, UsageBrief, UsagePlan,
};
pub use services::home::{
    CrewMember, HomeSummary, OutItem, PlanItem, Preferences, PreferencesUpdate, TodayItem,
};
pub use services::money::{
    Budget, CategoryTotal, DayTotal, MoneySummary, NewTransaction, NewTransactionItem, Transaction,
    TransactionItem,
};
pub use services::project_tools::{
    ContainerUsage, DataSource, DatabaseTable, DomainChange, DomainCheck, DomainSuggestion,
    ProjectDomain, ProjectLogs, ProjectShowcase, ProjectTraffic, QueryResult, RelayColumn,
    RelayOverview, RelaySetup, RelayTable, TrafficBucket, TrafficCounters, TransferOrg,
};
pub use services::projects::{
    BackendSetup, DeployOverview, IssueCounts, NewProjectIssue, ProjectDetail, ProjectFile,
    ProjectIssue, ProjectIssues, ProjectMembers, ProjectPerson, ProjectPublish,
};
pub use services::reminders::{
    NewReminder, Reminder, ReminderAction, RemindersOverview, ScheduledTask,
};
pub use services::shares::ShareLink;
pub use storage::models::{
    AgentActivity, DatabaseStatus, Draft, Message, MessageCursor, MessagePage, NotificationItem,
    OutboxEntry, OutboxOperation, OutboxState, Session, SessionFilter,
};
pub use storage::observation::{
    AgentActivityObservation, DraftObservation, DraftState, MessagesObservation,
    NotificationsObservation, OutboxObservation, SessionsObservation, UnreadCountObservation,
};
pub use storage::AccountStore;
pub use sync::ChatBinding;

/// The SDK's own version, as Cargo knows it.
#[uniffi::export]
pub fn sdk_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// `raw` as the realtime gateway spells a topic (`user`, `chat:<id>`, `project:<id>`, ids
/// lowercased), or `InvalidArgument`.
#[uniffi::export]
pub fn realtime_topic(raw: String) -> Result<String, NomiError> {
    network::realtime::normalize_topic(&raw)
}

/// Decodes a realtime payload (`payload_json` of an event on `topic`) as `SdkEvent::Realtime`
/// carries it. Never fails: what doesn't decode is `RealtimeEvent::Unknown`.
#[uniffi::export]
pub fn decode_realtime_event(topic: String, payload_json: String) -> RealtimeEvent {
    match serde_json::from_str(&payload_json) {
        Ok(payload) => network::events::decode(&topic, &payload),
        Err(_) => RealtimeEvent::Unknown {
            kind: None,
            payload_json,
        },
    }
}

/// A configuration with every optional setting at its default.
#[uniffi::export]
pub fn default_sdk_config(base_url: String, data_dir: String) -> SdkConfig {
    SdkConfig {
        base_url,
        data_dir,
        log_level: LogLevel::Info,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
    }
}
