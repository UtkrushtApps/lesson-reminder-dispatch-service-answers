-- Schema and seed data for the lesson reminder dispatch service.

CREATE TABLE IF NOT EXISTS lesson_reminders (
    id UUID PRIMARY KEY,
    student_id UUID NOT NULL,
    lesson_id UUID NOT NULL,
    student_email TEXT NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INT NOT NULL DEFAULT 0,
    last_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_lesson_reminders_due
    ON lesson_reminders (status, scheduled_at);

-- Seed data: a mix of due pending, future pending, already sent, and failed.
-- Three rows are due AND pending (alice, bob, carol) so a dispatch run handles 3.
INSERT INTO lesson_reminders
    (id, student_id, lesson_id, student_email, scheduled_at, status, attempts, last_error)
VALUES
    -- Due + pending (should be dispatched)
    ('11111111-1111-1111-1111-111111111111',
     '21111111-1111-1111-1111-111111111111',
     '31111111-1111-1111-1111-111111111111',
     'alice@example.com', NOW() - INTERVAL '10 minutes', 'pending', 0, NULL),
    ('11111111-1111-1111-1111-111111111112',
     '21111111-1111-1111-1111-111111111112',
     '31111111-1111-1111-1111-111111111112',
     'bob@example.com', NOW() - INTERVAL '5 minutes', 'pending', 0, NULL),
    ('11111111-1111-1111-1111-111111111113',
     '21111111-1111-1111-1111-111111111113',
     '31111111-1111-1111-1111-111111111113',
     'carol@example.com', NOW() - INTERVAL '1 minutes', 'pending', 0, NULL),
    -- Future + pending (should NOT be dispatched)
    ('11111111-1111-1111-1111-111111111114',
     '21111111-1111-1111-1111-111111111114',
     '31111111-1111-1111-1111-111111111114',
     'dave@example.com', NOW() + INTERVAL '2 hours', 'pending', 0, NULL),
    -- Already sent (must remain untouched)
    ('11111111-1111-1111-1111-111111111115',
     '21111111-1111-1111-1111-111111111115',
     '31111111-1111-1111-1111-111111111115',
     'erin@example.com', NOW() - INTERVAL '1 hours', 'sent', 1, NULL),
    -- Previously failed (must remain untouched by a pending-only run)
    ('11111111-1111-1111-1111-111111111116',
     '21111111-1111-1111-1111-111111111116',
     '31111111-1111-1111-1111-111111111116',
     'frank@example.com', NOW() - INTERVAL '30 minutes', 'failed', 2, 'previous transport error')
ON CONFLICT (id) DO NOTHING;
