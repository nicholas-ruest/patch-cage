#!/usr/bin/env bash
set -euo pipefail
for dep in ruvector ruflo metaharness rune synthlang openfeature otel; do printf "%s\tworkflow_completion=0\n" "$dep"; done > "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp}/ablations.tsv"
