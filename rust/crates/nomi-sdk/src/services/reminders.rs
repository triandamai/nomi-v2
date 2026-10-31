//! The Reminders page (`routes/reminders.rs`): the person's reminders, and agent work scheduled for
//! later, which can only be cancelled.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Services, NO_BODY};
use crate::error::NomiError;

const REMINDERS: &str = "reminders";
const SCHEDULED_TASKS: &str = "scheduled-tasks";
const CANCEL: &str = "cancel";
const DEFAULT_TIMEZONE: &str = "UTC";

/// `status` is `active`, `done` or `cancelled`; `created_by` is `user` or an agent.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due_at: String,
    pub recurrence: Option<String>,
    pub status: String,
    pub created_by: String,
    pub created_at: String,
}

/// Agent work that will run later.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ScheduledTask {
    pub id: String,
    pub label: String,
    pub run_at: String,
    pub recurrence: Option<String>,
    pub agent: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct RemindersOverview {
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default)]
    pub upcoming: Vec<Reminder>,
    #[serde(default)]
    pub past: Vec<Reminder>,
    #[serde(default)]
    pub scheduled_tasks: Vec<ScheduledTask>,
}

/// `due_at` is RFC 3339, or a local date-time read in the person's timezone. `recurrence` is
/// `daily`, `weekly` or `monthly`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, uniffi::Record)]
pub struct NewReminder {
    pub title: String,
    pub due_at: String,
    pub recurrence: Option<String>,
    pub notes: Option<String>,
}

/// What to do with a reminder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ReminderAction {
    Done,
    Cancel,
    /// Due again in `minutes`.
    Snooze {
        minutes: u32,
    },
}

#[derive(Serialize)]
struct ActionBody {
    action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    minutes: Option<u32>,
}

fn default_timezone() -> String {
    DEFAULT_TIMEZONE.to_owned()
}

impl Services {
    pub async fn reminders(&self) -> Result<RemindersOverview, NomiError> {
        self.get(&[REMINDERS], &[]).await
    }

    pub async fn create_reminder(&self, reminder: NewReminder) -> Result<Reminder, NomiError> {
        self.call_for(Method::POST, &[REMINDERS], Some(&reminder))
            .await
    }

    pub async fn act_on_reminder(
        &self,
        id: String,
        action: ReminderAction,
    ) -> Result<(), NomiError> {
        let body = match action {
            ReminderAction::Done => ActionBody {
                action: "done",
                minutes: None,
            },
            ReminderAction::Cancel => ActionBody {
                action: "cancel",
                minutes: None,
            },
            ReminderAction::Snooze { minutes } => ActionBody {
                action: "snooze",
                minutes: Some(minutes),
            },
        };
        self.call(Method::POST, &[REMINDERS, &id], Some(&body))
            .await
    }

    pub async fn cancel_scheduled_task(&self, id: String) -> Result<(), NomiError> {
        self.call(Method::POST, &[SCHEDULED_TASKS, &id, CANCEL], NO_BODY)
            .await
    }
}
