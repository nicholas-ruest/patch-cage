# Selection receipt — 2026-10-29

**Capability:** Patch Cage is a change-safety operations product that ingests a proposed patch, builds a repository-aware risk graph, executes bounded verification, routes human approval, and preserves a restart-durable evidence timeline before any release effect.

It is materially distinct from recent context, mandate, route, and contract products because its primary object is an executable software change and its outcome is a governed release decision backed by build, visual, policy, and precedent evidence. The earlier one-crate CLI state is rejected as noncompliant and is not the intended system.

Users are platform engineers, application maintainers, and security reviewers handling 10–200-file changes across protected repositories. Bounded contexts: intake/topology; analysis/simulation; governance/approval; memory/audit; release/observability.

The selected composition is `ruvnet/RuVector@fefc2ec966b6`, `ruvnet/ruflo@5d2be9374a38`, `ruvnet/metaharness@2f415bf47a7b`, `ruvnet/rune@92425512f0f7`, `ruvnet/SynthLang@cf16b42178b3`, `open-feature/rust-sdk@2a417c1fdb1c`, `open-telemetry/opentelemetry-rust@6860b49c0bab`. The frontier mechanism is claim-level provenance plus conformal abstention: Patch Cage refuses release when risk evidence is incomplete or calibration falls outside the accepted envelope. Success means all three browser journeys pass, denial creates no release event, a stored review survives backend restart, and every selected upstream emits correlated runtime evidence.

Ablations remove each upstream and compare completed-workflow rate, abstention accuracy, reproducible-build confidence, retrieval relevance, and p95 latency.
