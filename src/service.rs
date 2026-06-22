//! Service layer: orchestrates fetching due reminders, sending them, and
//! recording the outcome.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{DispatchSummary, Reminder, ReminderStatus};
use crate::errors::ServiceError;
use crate::repository::ReminderRepository;

/// Abstraction over the channel used to deliver a reminder.
#[async_trait]
pub trait ReminderSender: Send + Sync {
    async fn send(&self, reminder: &Reminder) -> Result<(), ServiceError>;
}

/// Find due reminders, send each one, and record the result.
///
/// Send failures are handled per reminder: a transport failure marks that
/// reminder as failed, increments the failed count, and does not stop the rest
/// of the batch. Database/recording errors still bubble up because the service
/// cannot accurately report persisted outcomes if it cannot update storage.
///
/// Each persisted status change is guarded by the repository so only rows that
/// are still `pending` and due are moved to a terminal state. If a fetched row
/// is no longer eligible by the time the service records the outcome, it is
/// counted as skipped.
pub async fn dispatch_due_reminders(
    repo: &ReminderRepository,
    sender: &dyn ReminderSender,
) -> Result<DispatchSummary, ServiceError> {
    let due = repo.fetch_due_reminders().await?;
    let mut summary = DispatchSummary::default();

    for reminder in due {
        let reminder_id = reminder.id;

        match sender.send(&reminder).await {
            Ok(()) => {
                let rows = repo
                    .transition_due_pending_to(reminder_id, ReminderStatus::Sent, None)
                    .await?;

                if rows == 1 {
                    summary.dispatched += 1;
                } else {
                    summary.skipped += 1;
                }
            }
            Err(send_error) => {
                let error_message = send_error.to_string();
                let rows = repo
                    .transition_due_pending_to(
                        reminder_id,
                        ReminderStatus::Failed,
                        Some(error_message.as_str()),
                    )
                    .await?;

                if rows == 1 {
                    summary.failed += 1;
                } else {
                    summary.skipped += 1;
                }
            }
        }
    }

    Ok(summary)
}

/// Simple in-memory sender used by examples and tests.
pub struct RecordingSender {
    fail_for_email: Option<String>,
}

impl RecordingSender {
    pub fn new(fail_for_email: Option<String>) -> Self {
        Self { fail_for_email }
    }
}

#[async_trait]
impl ReminderSender for RecordingSender {
    async fn send(&self, reminder: &Reminder) -> Result<(), ServiceError> {
        if let Some(target) = &self.fail_for_email {
            if target == &reminder.student_email {
                return Err(ServiceError::Send {
                    reminder_id: reminder.id,
                    message: "mock transport refused".to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Returns the well-known id used by seed data for diagnostics.
pub fn _example_id() -> Uuid {
    Uuid::nil()
}
