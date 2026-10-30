# Prototype acceptance

| Journey | Route/control | API | Middleware | Domain/application | Upstream/persistence | Evidence |
|---|---|---|---|---|---|---|
| primary | /intake Analyze and /review Approve | POST assessment/decision | auth, validation, correlation, timeout, tracing | pc-app across all counted crates | all five Ruvnet, two enterprise, RuVector/RVF write | Playwright [journey:primary] + trace |
| denial-recovery | /review Deny then Retry | POST decision/retry | policy denies before side effects | authz, policy, release | no downstream write on denial; recovery writes new revision | [journey:denial-recovery] |
| durable-retrieval | /audit Reload after restart | GET assessment | auth + correlation | audit query | RVF/RuVector reopen | [journey:durable-retrieval] |
