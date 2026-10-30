# Architecture

The React workbench calls a versioned Axum API. Middleware orders correlation, authentication, authorization, validation, rate limiting, timeout and tracing. pc-app composes intake, graph, risk, simulation, memory, approval and release crates. RuVector/RVF is the authoritative persistence/intelligence boundary; no service-owned relational database exists. External tools execute through pinned adapters and emit receipts.

See [crate map](crate-map.md), [integration matrix](integration-matrix.md), and [prototype acceptance](prototype-acceptance.md).
