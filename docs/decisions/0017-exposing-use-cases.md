# 0017. Exposing use cases

- **Status:** Accepted
- **Date:** 2026-11-12
- **Ticket:** STU-25

There is no generic layer between use cases and transports. `Grado` (ADR 0010) is the one
contract every inbound crate calls. Each inbound crate writes its bindings by hand, with its
framework's own tools, and owns everything its caller sees: names, request and response shapes,
and wording.

- **Authenticate once.** Each inbound crate authenticates the caller once per request, before any
  binding runs, and hands the bindings the `PlayerId`. On MCP that's `call_tool`.
- **An agent is a credential, not a role.** An agent connection acts for one player. Who may do
  what is decided in `domain`, through the use case, never in a binding. Roles are STU-29's.
- **No catalog of our own.** MCP lists its tools itself. An HTTP catalog, such as OpenAPI, is
  STU-83's choice.

**Consequences:**
- ADR 0004 rule 15 drops "Until STU-25 defines the mechanism": bindings stay hand-written. So do
  `Grado`'s forwarding methods (ADR 0010).
- Exposing a use case is one binding on that transport. The same use case can have another name
  and shape on each transport.
- If, once `inbound-http` exists, bindings repeat code the compiler can't check, a later record
  may add a binding trait inside each inbound crate. It doesn't add a registry shared by
  transports.
