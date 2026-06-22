//! Lesson reminder dispatch service.
//!
//! The service reads due reminders from PostgreSQL, sends each through a
//! `ReminderSender`, and records the outcome back to the database.

pub mod domain;
pub mod errors;
pub mod repository;
pub mod service;

pub use domain::{DispatchSummary, Reminder, ReminderStatus};
pub use errors::ServiceError;
pub use repository::ReminderRepository;
pub use service::{dispatch_due_reminders, ReminderSender};
