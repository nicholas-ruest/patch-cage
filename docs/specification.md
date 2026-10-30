# Specification

Patch Cage prevents an unreviewed or weakly evidenced software patch from reaching a release channel. A reviewer can create an assessment, inspect repository topology, policy findings, upstream verification, precedent matches and a proposed remediation, then approve or deny. The product retains the complete timeline across restart.

## Journeys

1. **Primary:** submit a deterministic synthetic patch, run all seven upstream responsibilities, inspect correlated evidence, approve and stage a release.
2. **Denial and recovery:** a protected-path patch submitted by an unauthorized actor is denied before sandbox or release effects; a reviewer corrects scope and reruns.
3. **Durable retrieval:** restart the backend, reopen the assessment by ID, and render the same append-only audit timeline.

SLO: readiness under 10 seconds, analysis p95 under 2 seconds excluding bounded external tools, zero forbidden effects after denial, and exact replay after restart.
