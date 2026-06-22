//! Recoverable error types for the dispatch service.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("send failed for reminder {reminder_id}: {message}")]
    Send {
        reminder_id: uuid::Uuid,
        message: String,
    },

    #[error("invalid reminder status: {0}")]
    InvalidStatus(String),
}
