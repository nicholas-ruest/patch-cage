#!/usr/bin/env bash
set -euo pipefail
cargo test -p pc-ruvector --locked
cargo test -p pc-openfeature --locked
cargo test -p pc-telemetry --locked
mkdir -p "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp}"
printf '%s\n' '{"source":"actual-test-boundaries","ruvnet":{"ruvnet/RuVector":"executed","ruvnet/ruflo":"not-executed-local-adapter-only","ruvnet/metaharness":"not-executed-local-adapter-only","ruvnet/rune":"not-executed-local-adapter-only","ruvnet/SynthLang":"not-executed-local-adapter-only"},"enterprise":{"open-feature/rust-sdk":"executed","open-telemetry/opentelemetry-rust":"executed"},"verdict":"FAIL_RUVNET_1_OF_5"}' > "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp}/integration-trace.json"
echo 'RUVNET_EXECUTED 1/5; ENTERPRISE_EXECUTED 2/2' >&2
exit 1
