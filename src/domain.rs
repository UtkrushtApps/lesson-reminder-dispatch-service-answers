//! Domain models for the reminder dispatch service.
//!
//! The reminder lifecycle is represented as a typed domain enum and explicitly
//! mapped to/from the string values stored in the `lesson_reminders.status`
//! column.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::ServiceError;

/// Lifecycle state of a reminder row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReminderStatus {
    Pending,
    Sent,
    Failed,
}

impl ReminderStatus {
    /// Database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            ReminderStatus::Pending => "pending",
            ReminderStatus::Sent => "sent",
            ReminderStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for ReminderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReminderStatus {
    type Err = ServiceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(ReminderStatus::Pending),
            "sent" => Ok(ReminderStatus::Sent),
            "failed" => Ok(ReminderStatus::Failed),
            other => Err(ServiceError::InvalidStatus(other.to_string())),
        }
    }
}

impl TryFrom<String> for ReminderStatus {
    type Error = ServiceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl TryFrom<&str> for ReminderStatus {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// A single reminder loaded from storage.
#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: Uuid,
    pub student_id: Uuid,
    pub lesson_id: Uuid,
    pub student_email: String,
    pub scheduled_at: DateTime<Utc>,
    pub status: ReminderStatus,
    pub attempts: i32,
}

/// Result summary returned by a dispatch run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DispatchSummary {
    pub dispatched: usize,
    pub failed: usize,
    pub skipped: usize,
}
