#!/usr/bin/env bash
set -euo pipefail
mkdir -p /tmp/patch-cage
export PORT="${PORT:-18080}"
cargo run -p pc-api > /tmp/patch-cage/backend.log 2>&1 & echo $! > /tmp/patch-cage/backend.pid
npm --prefix frontend run dev > /tmp/patch-cage/frontend.log 2>&1 & echo $! > /tmp/patch-cage/frontend.pid
