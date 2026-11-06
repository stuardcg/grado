# 0009. Async ports

- **Status:** Accepted
- **Date:** 2026-11-06
- **Ticket:** STU-43

Ports are async traits written with `async-trait`, and use cases take them as `&dyn Port`. Tests
run on `tokio`, which `application` may use as a dev-dependency only; this adds to ADR 0004's
crate table.

**Consequences:** a database or network store implements a port without blocking, and use cases
are async from the start, so adding one doesn't rewrite them.
