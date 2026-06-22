//! Behavior tests for the reminder dispatch service.
//!
//! These require the PostgreSQL container from docker-compose to be running.

use lesson_reminder_dispatch_service::repository::ReminderRepository;
use lesson_reminder_dispatch_service::service::{dispatch_due_reminders, RecordingSender};
use sqlx::postgres::PgPoolOptions;

const DATABASE_URL: &str =
    "postgres://reminder_user:reminder_pass@127.0.0.1:5432/reminder_db";

async fn reset_seed(repo: &ReminderRepository) {
    // Restore the seed table to a known state before each test.
    sqlx::query("TRUNCATE lesson_reminders")
        .execute(repo.pool())
        .await
        .expect("truncate");

    sqlx::query(include_str!("../init_database.sql"))
        .execute(repo.pool())
        .await
        .ok();

    // The seed file also creates the table/indexes; re-running is harmless
    // because of IF NOT EXISTS, but the data inserts must succeed.
}

async fn repo() -> ReminderRepository {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DATABASE_URL)
        .await
        .expect("connect to postgres");
    ReminderRepository::new(pool)
}

#[tokio::test]
async fn sends_each_due_reminder_once() {
    let repo = repo().await;
    reset_seed(&repo).await;

    let sender = RecordingSender::new(None);
    let summary = dispatch_due_reminders(&repo, &sender)
        .await
        .expect("dispatch run");

    // There are 3 due pending reminders in the seed data.
    assert_eq!(summary.dispatched, 3, "expected 3 due reminders dispatched");
    assert_eq!(summary.failed, 0);

    let sent = repo.count_with_status("sent").await.expect("count sent");
    assert_eq!(sent, 3, "all due reminders should now be sent");
}

#[tokio::test]
async fn second_run_is_idempotent() {
    let repo = repo().await;
    reset_seed(&repo).await;

    let sender = RecordingSender::new(None);
    let _ = dispatch_due_reminders(&repo, &sender)
        .await
        .expect("first run");

    let second = dispatch_due_reminders(&repo, &sender)
        .await
        .expect("second run");

    assert_eq!(
        second.dispatched, 0,
        "a second run over unchanged data must dispatch nothing"
    );
}

#[tokio::test]
async fn one_failure_does_not_abort_batch() {
    let repo = repo().await;
    reset_seed(&repo).await;

    // This email belongs to one of the due reminders in the seed data.
    let sender = RecordingSender::new(Some("bob@example.com".to_string()));
    let summary = dispatch_due_reminders(&repo, &sender)
        .await
        .expect("dispatch run should not error out");

    assert_eq!(summary.failed, 1, "the failing send should be counted");
    assert_eq!(
        summary.dispatched, 2,
        "the other due reminders should still be sent"
    );

    let failed = repo.count_with_status("failed").await.expect("count failed");
    assert_eq!(failed, 1, "the failing reminder should be recorded as failed");
}
