//! Repository layer: all PostgreSQL access for reminders lives here.

use chrono::Utc;
use sqlx::{postgres::PgPool, Row};
use uuid::Uuid;

use crate::domain::{Reminder, ReminderStatus};
use crate::errors::ServiceError;

#[derive(Clone)]
pub struct ReminderRepository {
    pool: PgPool,
}

impl ReminderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Fetch all reminders that are due to be sent right now.
    pub async fn fetch_due_reminders(&self) -> Result<Vec<Reminder>, ServiceError> {
        let now = Utc::now();
        let rows = sqlx::query(
            "SELECT id, student_id, lesson_id, student_email, scheduled_at, status, attempts \
             FROM lesson_reminders \
             WHERE status = $1 AND scheduled_at <= $2 \
             ORDER BY scheduled_at ASC",
        )
        .bind(ReminderStatus::Pending.as_str())
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let mut reminders = Vec::with_capacity(rows.len());
        for row in rows {
            let status: String = row.get("status");
            reminders.push(Reminder {
                id: row.get("id"),
                student_id: row.get("student_id"),
                lesson_id: row.get("lesson_id"),
                student_email: row.get("student_email"),
                scheduled_at: row.get("scheduled_at"),
                status: ReminderStatus::try_from(status)?,
                attempts: row.get("attempts"),
            });
        }

        Ok(reminders)
    }

    /// Guardedly move a due, pending reminder to a terminal status.
    ///
    /// The `WHERE status = 'pending' AND scheduled_at <= now()` guard is what
    /// makes stale fetches safe: if another dispatcher or process already
    /// handled the reminder, this update affects zero rows and the service can
    /// count that item as skipped instead of reporting it as dispatched/failed.
    pub async fn transition_due_pending_to(
        &self,
        reminder_id: Uuid,
        new_status: ReminderStatus,
        last_error: Option<&str>,
    ) -> Result<u64, ServiceError> {
        let now = Utc::now();
        let result = sqlx::query(
            "UPDATE lesson_reminders \
             SET status = $1, last_error = $2, attempts = attempts + 1 \
             WHERE id = $3 AND status = $4 AND scheduled_at <= $5",
        )
        .bind(new_status.as_str())
        .bind(last_error)
        .bind(reminder_id)
        .bind(ReminderStatus::Pending.as_str())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Update a reminder's status using the guarded domain transition.
    ///
    /// This method is kept as a small compatibility wrapper around the typed
    /// repository API. New orchestration code should prefer
    /// `transition_due_pending_to` so callers cannot pass arbitrary status
    /// strings.
    pub async fn update_status(
        &self,
        reminder_id: Uuid,
        new_status: &str,
        last_error: Option<&str>,
    ) -> Result<u64, ServiceError> {
        let new_status = ReminderStatus::try_from(new_status)?;
        self.transition_due_pending_to(reminder_id, new_status, last_error)
            .await
    }

    /// Count reminders currently in a given status (test/diagnostic helper).
    pub async fn count_with_status(&self, status: &str) -> Result<i64, ServiceError> {
        let row = sqlx::query("SELECT COUNT(*) AS c FROM lesson_reminders WHERE status = $1")
            .bind(status)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("c"))
    }

    /// Count reminders currently in a typed status.
    pub async fn count_with_reminder_status(
        &self,
        status: ReminderStatus,
    ) -> Result<i64, ServiceError> {
        self.count_with_status(status.as_str()).await
    }
}
