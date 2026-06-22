#!/usr/bin/env bash
set -euo pipefail

cd /root/task

echo "==> [1/5] Fetching Rust dependencies via Cargo..."
cargo fetch

echo "==> [2/5] Starting PostgreSQL container..."
docker compose up -d

echo "==> [3/5] Waiting for PostgreSQL to become healthy..."
attempts=0
max_attempts=30
until docker compose exec -T postgres pg_isready -U reminder_user -d reminder_db >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [ "$attempts" -ge "$max_attempts" ]; then
        echo "PostgreSQL did not become healthy in time." >&2
        docker compose logs postgres >&2 || true
        exit 1
    fi
    echo "    ...waiting ($attempts/$max_attempts)"
    sleep 2
done
echo "    PostgreSQL is ready."

echo "==> [4/5] Building the starter project (all targets)..."
cargo build --all-targets

echo "==> [5/5] Compiling tests without running them..."
cargo test --no-run

echo "Environment is ready. Starter compiles successfully."
exit 0
