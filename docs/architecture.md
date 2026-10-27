# Architecture

## The loop

1. The player connects Claude Code once, with a personal access token from the website.
2. In her project she runs `/mcp__grado__checkpoint`. The MCP prompt returns the
   **briefing**: the core coach (`content/core.md`), the current checkpoint's `brief.md`, the
   previous checkpoint's handoff note, and ideas she parked for this checkpoint.
3. The coach calls `get_current_check` and sees **one** criterion: its title, what to collect,
   and whether it needs raw output or her words. Never the check itself.
4. The coach calls `submit_evidence`. The server judges it and answers pass/fail with feedback
   that names what is missing without giving the answer. Out-of-order submissions are refused.
5. When every criterion has passed, `finish_checkpoint` stores the handoff note and returns a
   one-time stamp (stored only as a hash, expires after 24 h, a new one voids the old one).
6. She enters the stamp on the website. Redeeming completes the checkpoint and unlocks the next.

Nothing in this loop trusts the agent's word: the server decides what counts as done.

## Crates

```
grado-server ──▶ grado-domain   (content, journey, checks; pure)
      │
      └───────▶ grado-store    (Store trait + SQLite)
```

- **`grado-domain`** knows the content format and the rules: which level is active
  (`Journey`), how to judge evidence (`checks::evaluate`), how to assemble a briefing. Rubric
  grading is behind the `RubricGrader` trait so the domain never makes network calls.
- **`grado-store`** is the only place with SQL. The rest of the code sees the `Store` trait.
- **`grado-server`** wires both together. `trail::Trail` is the service layer: every rule-level
  operation (submit, finish, redeem, open guide...) lives there once, and both the website
  (`http/`) and the MCP tools (`mcp.rs`) call it.

## Evidence checks

Defined per criterion in `level.toml` (`kind = ...`):

| Kind | Judges | Used for |
|---|---|---|
| `present` | Something non-trivial was submitted | git log |
| `contains_all` | Every needle appears | PDF header, `failed` in a deliberately broken test run |
| `key_values` | `key: value` lines match hidden values, any order | Category counts from her app |
| `pytest` | Enough passing tests, none failing | Safety net, refactoring |
| `rubric` | A hidden rubric, graded by Claude (offline keyword fallback) | Her notes, plans, explanations |
| `all` | Every inner check | Combining the above |

`key_values` makes code checkpoints tamper-evident: the expected counts depend on edge cases in
the sample data (duplicate emails in different case, a rejected row that *looks* like a
duplicate, an impossible date, birthdays on race day). The agent can't know them in advance.

## Auth

- **Website:** session cookie (`HttpOnly`, `SameSite=Lax`, hashed in the database). v1 ships a
  development sign-in only. A real identity provider replaces `dev_login` and ends by calling
  `http::auth::start_session` for the verified email; nothing else changes.
- **MCP:** `Authorization: Bearer grd_...` resolved by `mcp::require_bearer` before rmcp sees the
  request. Tokens are created and revoked on the Connect page and stored as hashes.
- **Game master:** emails in `GRADO_ADMIN_EMAILS`.

## MCP transport

Streamable HTTP via `rmcp`, stateless (`legacy_session_mode = false`, JSON responses, no SSE
keep-alive). Each request stands alone, which fits serverless hosting. `GRADO_ALLOWED_HOSTS`
must include the public hostname, or rmcp rejects requests (DNS-rebinding protection).

## Content versioning

`Catalog::version` is a hash of every file in `content/`. Progress rows record the version a
checkpoint was started and completed on, so you can trace a run back to the exact content it
was played with after you tune things.
