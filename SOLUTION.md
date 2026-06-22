# Solution Steps

1. Model reminder status in the domain layer by changing `Reminder.status` from `String` to `ReminderStatus` and implementing explicit conversions between `ReminderStatus` and the database strings `pending`, `sent`, and `failed`.

2. Update repository reads so `fetch_due_reminders` queries only pending, due rows and parses the returned status string into the domain enum, returning `ServiceError::InvalidStatus` for unknown database values.

3. Move guarded persistence logic into the repository by adding a typed `transition_due_pending_to` method. The SQL update should set the terminal status, record `last_error`, increment `attempts`, and include `WHERE id = ... AND status = 'pending' AND scheduled_at <= ...` so stale or already-handled rows affect zero rows.

4. Keep a compatibility `update_status` wrapper if needed, but have it parse the provided status string into `ReminderStatus` and delegate to the guarded typed repository method.

5. Rewrite `dispatch_due_reminders` as orchestration only: fetch due reminders, loop over them, call the sender, then record either `Sent` or `Failed` using the repository transition method.

6. Handle `sender.send` errors inside the loop instead of using `?`: convert the send error to a string, attempt to mark that reminder as `Failed`, increment `summary.failed` if the guarded update affected one row, or `summary.skipped` if it affected zero rows.

7. For successful sends, mark the reminder as `Sent`; increment `summary.dispatched` only when the guarded update affects one row, otherwise increment `summary.skipped`.

8. Continue processing the rest of the batch after individual send failures. Still return database/update errors with `?`, because without persistence the service cannot accurately report stored outcomes.

9. Run `cargo fmt`, build the project, start PostgreSQL with Docker Compose, and run the integration tests to confirm due reminders are sent once, the second run dispatches nothing, and one failing send does not abort the batch.

