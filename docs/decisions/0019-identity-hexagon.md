# 0019. Identity as its own hexagon

- **Status:** Accepted
- **Date:** 2026-11-13
- **Ticket:** STU-29

Identity is a second hexagon next to the game's, under ADR 0004's rules: accounts, identity links,
sessions, agent connections and sign-in with providers. Its core is `identity-domain` and
`identity-application`, with its own handle, `Identity`. The same servers run both hexagons.

- **Neither core depends on the other.** Identity's rules (an identity belongs to one account, the
  session lifetimes) aren't game rules, and ADR 0004 gives `domain` only game rules.
- **Identity says who; the game says whether.** Identity authenticates the caller and returns an
  `AccountId`. The inbound crate turns it into a `PlayerId` with the same value; both are
  `shared::id::Id<T>`, so the value crosses and the types don't mix. Identity's own use cases
  (agent connections, sign-out) take the `AccountId`; only the game's take the `PlayerId`. Roles
  stay in the game.
- **A player appears on first use.** The game treats an authenticated id it hasn't seen as a new
  player with the `player` role, so signing up needs no message between the hexagons.
- **Agent connections are identity's.** `authenticate_agent_connection`, its port and `AgentToken`
  move to `identity-application`.

**Consequences:**
- Every ADR 0004 rule for `domain` and `application` applies to `identity-domain` and
  `identity-application`, at their own paths.
- ADR 0004 changes:
  - its crate table gains `identity-domain` and `identity-application`, with the rows of `domain`
    and `application`;
  - inbound, outbound and server crates may depend on both cores;
  - rule 24's authentication use cases are identity's;
  - "Where a change goes" sends an identity rule to `identity-domain`, an identity use case to
    `identity-application/src/use_cases/`, and an identity port to
    `identity-application/src/ports/`.
- ADR 0010 changes: each core has its own handle. The composition root builds both and hands both
  to the inbound crates.
- Google and GitHub are reached through one new crate, `outbound-oauth`. `outbound-memory`, and
  later the database adapter, implement ports of both cores.
- If a hosted or self-hosted auth server comes back (ADR 0018), only the identity hexagon changes.
