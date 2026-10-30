# Traceability

| Requirement | UI | API | Crates | Evidence |
|---|---|---|---|---|
| analyze patch | /intake Analyze | POST /v1/assessments | ingest → graph → risk → app | primary trace |
| deny safely | /review Deny | POST /v1/assessments/:id/decision | authz → policy → release | denial trace |
| retrieve | /audit | GET /v1/assessments/:id | audit → RVF/RuVector | restart trace |
| integration health | /integrations | GET /v1/integrations | five Ruvnet + two enterprise adapters | composition report |
