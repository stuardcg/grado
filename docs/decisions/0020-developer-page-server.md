# 0020. A server for the developer page

- **Status:** Accepted
- **Date:** 2026-11-13
- **Ticket:** STU-29

Players use the **game page**; developers use the **developer page**, a web app of its own for
content, players' state and developer roles. The developer page has its own deployable,
`developer-server`, with its own inbound crate, `inbound-developer`, at its own origin. Only
`inbound-developer` binds developer use cases; `inbound-http` and `inbound-mcp` never do. Every
server runs the same two hexagons (ADR 0019) and the same storage adapters.

**Consequences:**
- Developer powers stay off the game page's server and the MCP server by construction, so a leaked
  agent token can't reach them. The developer page can get its own limits, logs and network
  restrictions.
- Its own origin gives it its own session cookie, and each session records the door that issued
  it (ADR 0018), so the developer page's shorter lifetime needs no role in identity.
- ADR 0014 changes: a deployable serves one door, not one transport. `mcp-server` serves agents,
  `http-server` the game page and `developer-server` the developer page. A new door is a new
  `inbound-<door>` crate and a new `<door>-server` crate, named after who uses it; the two
  existing servers keep their names. The frontends are `web/game/` and `web/developer/`, not
  `web/devtools/`.
- `inbound-http` and `inbound-developer` both need sign-in, cookie, `Origin` and CSP handling: code
  two adapters share, which ADR 0004's "Shared code" table sends to a record. As ADR 0014 did for
  shared startup code, that record comes when `developer-server` is added.
- ADR 0017's "no registry shared by transports" covers the two HTTP inbound crates too.
- "The app" and "devtools" in ADR 0006 and the domain docs mean the game page and the developer
  page.
- STU-83 builds `http-server` for the game page only. Each origin needs its own callback with
  Google and GitHub.
