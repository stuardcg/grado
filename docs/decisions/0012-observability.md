# 0012. Observability

- **Status:** Accepted
- **Date:** 2026-11-09
- **Ticket:** STU-33

Adapters and `app` log with `tracing`; `domain` and `application` never log. An unexpected
failure is logged exactly once, by the inbound adapter, where it maps the failure to its
transport's error, and with the failure's cause. `PortError` carries that cause, and outbound
adapters return their failures without logging them. Each inbound adapter opens one span per call
in one place, never in each binding. For MCP, that's `call_tool` in `bindings.rs`, with the
request id, the tool name and, once authenticated, the player id. A request keeps the
`x-request-id` it was sent with, or gets a new uuid, and the response returns it. Logs go to
stdout as JSON or pretty text, at a `tracing` filter level, both set by configuration (ADR 0011).
Tokens, player-written text and request bodies are never logged.

**Consequences:** a port failure the caller sees as "unavailable" always has a log line that says
why, with its request id. Two `PortError`s compare equal by kind, because the cause is only for
the log. Expected refusals, such as a missing handoff note, aren't logged. OpenTelemetry and X-Ray
wait for STU-37; they add one `tracing` layer in `app` and change no call site. The stored history
of agent reports is data (STU-28), not a log.
