# Crate map

All crates have a typed API, unit/failure tests, a caller in pc-app or another listed crate, and a mapped workflow.

| Crate | Bounded context | Responsibility | Caller | Acceptance |
|---|---|---|---|---|
| pc-types | foundation | Shared typed patch, actor, risk and evidence contracts | pc-app / downstream service | unit test plus journey trace |
| pc-ingest | intake | Unified-diff parsing, file limits and binary rejection | pc-app / downstream service | unit test plus journey trace |
| pc-repository | intake | Repository topology and protected-path model | pc-app / downstream service | unit test plus journey trace |
| pc-change-graph | analysis | Dependency and ownership blast-radius graph | pc-app / downstream service | unit test plus journey trace |
| pc-policy | governance | Declarative policy evaluation with explainable findings | pc-app / downstream service | unit test plus journey trace |
| pc-authz | governance | Role and tenant authorization decisions | pc-app / downstream service | unit test plus journey trace |
| pc-approval | governance | Two-person approval state machine and revocation | pc-app / downstream service | unit test plus journey trace |
| pc-sandbox-plan | simulation | Bounded command and resource execution plans | pc-app / downstream service | unit test plus journey trace |
| pc-sandbox-runner | simulation | Cancelable sandbox execution with timeout receipts | pc-app / downstream service | unit test plus journey trace |
| pc-risk-engine | analysis | Calibrated risk aggregation and uncertainty | pc-app / downstream service | unit test plus journey trace |
| pc-ruvector | memory | RuVector-backed precedent memory and retrieval | pc-app / downstream service | unit test plus journey trace |
| pc-rvf-ledger | memory | Append-only RVF-compatible evidence envelopes | pc-app / downstream service | unit test plus journey trace |
| pc-ruflo | integration | Ruflo workflow orchestration boundary | pc-app / downstream service | unit test plus journey trace |
| pc-metaharness | integration | MetaHarness repository scoring boundary | pc-app / downstream service | unit test plus journey trace |
| pc-rune | integration | Rune reproducible-build verification boundary | pc-app / downstream service | unit test plus journey trace |
| pc-synthlang | integration | SynthLang intent compression/classification boundary | pc-app / downstream service | unit test plus journey trace |
| pc-openfeature | integration | OpenFeature rollout decision boundary | pc-app / downstream service | unit test plus journey trace |
| pc-telemetry | runtime | OpenTelemetry spans and correlation propagation | pc-app / downstream service | unit test plus journey trace |
| pc-review | workflow | Review queue, comments and remediation workflow | pc-app / downstream service | unit test plus journey trace |
| pc-release | workflow | Fail-closed release gate and no-effect denial | pc-app / downstream service | unit test plus journey trace |
| pc-audit | audit | Durable timeline query and restart recovery | pc-app / downstream service | unit test plus journey trace |
| pc-app | application | Application orchestration across all bounded contexts | pc-app / downstream service | unit test plus journey trace |
| pc-api | interface | Versioned Axum API, middleware and service entrypoint | binary entrypoint | unit test plus journey trace |
