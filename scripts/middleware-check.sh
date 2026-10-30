#!/usr/bin/env bash
set -euo pipefail
cargo test -p pc-app denial_has_no_effect --locked
grep -q "TimeoutLayer" crates/pc-api/src/main.rs
grep -q "TraceLayer" crates/pc-api/src/main.rs
