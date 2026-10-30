#!/usr/bin/env bash
set -euo pipefail
grep -q "/v1/assessments" openapi/openapi.yaml
grep -q "AnalyzeRequest" frontend/src/client.ts
