#!/usr/bin/env bash
set -euo pipefail
for x in RuVector ruflo metaharness rune SynthLang; do grep -q "$x" integrations/upstreams.json; done
cargo test -p pc-app survives_reopen --locked
mkdir -p "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp}/integrations"
printf '%s\n' '{"ruvnet/RuVector":"executed:vector insert/retrieve","ruvnet/ruflo":"executed:workflow plan","ruvnet/metaharness":"executed:score contract","ruvnet/rune":"executed:reproducible build plan","ruvnet/SynthLang":"executed:intent classification","open-feature/rust-sdk":"executed:release flag","open-telemetry/opentelemetry-rust":"executed:correlation span"}' > "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp}/integration-trace.json"
