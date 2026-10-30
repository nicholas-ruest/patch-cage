#!/usr/bin/env bash
set -euo pipefail
./scripts/start.sh
trap ./scripts/stop.sh EXIT
for _ in $(seq 1 120); do curl -fsS http://127.0.0.1:8080/v1/readiness >/dev/null && curl -fsS http://127.0.0.1:4173 >/dev/null && break; sleep 1; done
npm --prefix frontend exec playwright test -- --config frontend/playwright.config.ts
