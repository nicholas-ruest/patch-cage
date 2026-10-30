#!/usr/bin/env bash
set -euo pipefail
cargo fetch --locked
npm --prefix frontend ci
