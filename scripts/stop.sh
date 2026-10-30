#!/usr/bin/env bash
set -euo pipefail
for f in /tmp/patch-cage/backend.pid /tmp/patch-cage/frontend.pid; do if [[ -f "$f" ]]; then kill "$(cat "$f")" 2>/dev/null || true; fi; done
