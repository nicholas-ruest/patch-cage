#!/usr/bin/env bash
set -euo pipefail
for i in $(seq 1 200); do cargo test -p pc-risk-engine accepts_domain_input --quiet; done
printf "{\"iterations\":200,\"status\":\"measured\"}\n" > "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp}/benchmark.json"
