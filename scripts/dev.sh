#!/usr/bin/env bash
# Starts the full local dev stack in one command: Postgres + EMQX (docker compose), the
# backend (nomi-orchestrator, with the turn-processing worker embedded by default — see
# RUN_WORKER_INLINE in backend/src/main.rs), and the frontend (vite dev). Ctrl+C stops everything.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cleanup() {
	echo ""
	echo "==> shutting down"
	kill 0
}
trap cleanup EXIT INT TERM

echo "==> starting Postgres + EMQX (docker compose)"
(cd backend && docker compose up -d)

echo "==> starting backend (nomi-orchestrator; worker runs embedded — set RUN_WORKER_INLINE=false to run it separately via 'cargo run --bin worker')"
(cd backend && cargo run --bin nomi-orchestrator 2>&1 | sed -u 's/^/[backend] /') &

echo "==> starting frontend (vite dev)"
(cd frontend && npm run dev 2>&1 | sed -u 's/^/[frontend] /') &

wait
