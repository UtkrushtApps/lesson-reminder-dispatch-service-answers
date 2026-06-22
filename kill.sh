#!/usr/bin/env bash
set -uo pipefail

echo "==> Starting cleanup of the lesson reminder dispatch environment..."

if [ -d /root/task ]; then
    echo "==> Changing into /root/task"
    cd /root/task
else
    echo "==> /root/task not found; continuing cleanup anyway"
fi

echo "==> Stopping Docker containers..."
docker compose down || true

echo "==> Removing Docker volumes..."
docker compose down -v || true
docker volume rm task_reminder_pgdata || true
docker volume rm reminder_pgdata || true

echo "==> Removing task-specific Docker networks..."
docker network rm task_default || true

echo "==> Force-removing task-specific Docker images (if any)..."
docker rmi -f lesson-reminder-dispatch-service || true

echo "==> Running final Docker system prune..."
docker system prune -a --volumes -f || true

echo "==> Removing the task directory..."
rm -rf /root/task || true

echo "Cleanup completed successfully!"
