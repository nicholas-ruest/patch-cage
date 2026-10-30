#!/usr/bin/env bash
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EVIDENCE="${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
GATE="$ROOT/scripts/dream-machine-product-gate.mjs"
overall=0
for suite in rust frontend api middleware integrations benchmark ablations containers browser; do
  script="$ROOT/scripts/${suite}-check.sh"
  case "$suite" in
    benchmark) script="$ROOT/scripts/benchmark.sh" ;;
    ablations) script="$ROOT/scripts/ablations.sh" ;;
  esac
  node "$GATE" record "$ROOT" "$EVIDENCE" "$suite" -- "$script" || overall=1
done
node "$ROOT/scripts/make-evidence.mjs" "$ROOT" "$EVIDENCE" || overall=1
node "$GATE" verify "$ROOT" "$EVIDENCE" "$(git -C "$ROOT" rev-parse HEAD)" || overall=1
exit "$overall"
