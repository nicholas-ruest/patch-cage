#!/usr/bin/env bash
set -euo pipefail
docker build -t patch-cage:${DREAM_MACHINE_SOURCE_SHA:-local} .
docker compose config >/dev/null
