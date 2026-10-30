#!/usr/bin/env bash
set -euo pipefail
rm -f /tmp/patch-cage/events.jsonl
curl -fsS -X POST http://127.0.0.1:8080/v1/assessments -H "content-type: application/json" -H "x-patch-cage-role: reviewer" -d @fixtures/seed.json
