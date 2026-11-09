# 0010. Application handle

- **Status:** Accepted
- **Date:** 2026-11-09
- **Ticket:** STU-46

Inbound adapters call use cases through one application handle, `Grado`, in
`application/src/lib.rs`. It holds every port privately and has one method per use case that only
forwards to that use case's `run`. The composition root, `app/src/bin/<runtime>.rs`, is the only
place that chooses adapters: it builds the outbound adapters, builds `Grado` from them, and hands
`Grado` to the inbound adapters.

**Consequences:** the compiler enforces ADR 0004 rule 14, because an inbound adapter can't reach a
private port. Use cases keep `run(&dyn Port, …, input)`, so each still names the ports it needs and
its tests pass fakes directly. A new use case needs a `Grado` method before any binding can call
it. Port value types an inbound adapter needs, such as `AgentToken`, are re-exported from
`application`. STU-25 may generate the forwarding methods.
