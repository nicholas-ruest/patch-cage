#!/usr/bin/env bash
set -euo pipefail
export PORT="${PORT:-18080}"
./scripts/start.sh
trap ./scripts/stop.sh EXIT
for _ in $(seq 1 120); do
  if curl -fsS "http://127.0.0.1:${PORT}/v1/readiness" >/dev/null && curl -fsS http://127.0.0.1:4173 >/dev/null; then
    ready=1
    break
  fi
  sleep 1
done
test "${ready:-0}" = 1
npm --prefix frontend exec playwright test -- --config playwright.config.ts
