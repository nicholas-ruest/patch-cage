#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EVIDENCE="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
GATE="$ROOT/scripts/dream-machine-product-gate.mjs"
for suite in rust frontend api middleware integrations benchmark ablations containers browser; do node "$GATE" record "$ROOT" "$EVIDENCE" "$suite" -- "$ROOT/scripts/${suite}-check.sh"; done
node "$ROOT/scripts/make-evidence.mjs" "$ROOT" "$EVIDENCE"
node "$GATE" verify "$ROOT" "$EVIDENCE" "$(git -C "$ROOT" rev-parse HEAD)"
