# 0018. Sign-in, agent tokens and roles

- **Status:** Accepted
- **Date:** 2026-11-13
- **Ticket:** STU-29

People sign in with GitHub or Google, through Grado's own code, with no passwords and no auth
provider. Agents use personal access tokens. The roles are `player` and `developer`, checked on
every call in `domain`. There is no dev mode.

**Sign-in.**

- GitHub (OAuth, then its user API) and Google (OIDC) at launch, each through an app Grado
  registered, built with the `openidconnect` and `oauth2` crates. Microsoft and Apple wait until
  someone asks. A hosted or self-hosted auth server comes back only with passwords, two-factor,
  passkeys or company sign-in (SAML).
- An account links identities, each `(provider, subject)`; nothing is keyed on email. Grado keeps
  the provider's id and a display name: no email, no avatar, none of the provider's tokens.
- A second provider is linked only while signed in. An identity another account owns is refused,
  never moved, and the last identity can't be unlinked. There is no recovery fallback.
- The flow checks `state`, PKCE and the nonce, and the ID token's issuer, audience and expiry. Each
  provider has its own callback, and the sign-in cookie names the provider it expects. Redirect
  URIs match exactly (GitHub's wildcard matching is off), and the address to return to after
  sign-in is only a path on the door's own origin.

**Sessions.**

- Server-side: a 256-bit random value in a `__Host-` cookie, `HttpOnly`, `Secure`,
  `SameSite=Lax`. The database keeps only its SHA-256 hash. Every sign-in issues a new value;
  sign-out is a POST that deletes it.
- Each session records the door that issued it, and identity refuses it at any other door. Game
  page: 7 days idle, 30 days at most. Developer page: 1 day idle, 7 days at most.
- A request that changes something must carry its door's own exact `Origin`; a GET changes
  nothing. Pages send a Content-Security-Policy without inline scripts, and names from providers
  are untrusted text.
- Each page and its API share one origin on Grado's domain.
- The idle limits of sessions and agent tokens rest on a "last used" time, written at most once
  an hour as a touch. A touch saves no loaded state, so ADR 0004 rule 25's version check doesn't
  apply to it.

**Agent tokens.**

- A personal access token, created on the game page and sent as `Authorization: Bearer`: a
  `grado_` prefix and a 256-bit random value, shown once. Grado keeps its SHA-256 hash, a name the
  player gives it, when it was created and last used, and whether it's revoked.
- It expires after 90 days without use, with no fixed limit, and a revoked one is refused at once
  (E8).
- MCP OAuth waits until a player needs a chat client's connector (claude.ai, ChatGPT). Grado's own
  authorization server or WorkOS Standalone Connect is chosen then.

**Roles.**

- `player`: every account, from first use. The game page and every MCP tool, on their own data
  only. `developer`: also the developer page, on any player. An agent is not a role (ADR 0017).
- A developer use case loads the caller's role through a game port on every call and asks
  `domain`. A player use case takes only the caller's `PlayerId`, so it can't name anyone else.
- The bootstrap command grants the first developer through the game's `grant_first_developer`
  use case, which `domain` refuses once any developer exists. After that, developers grant and
  remove the role on the developer page; the last developer can't be removed.
- Every developer-page call logs the developer, the target player and the use case.

**No dev mode.** Nothing signs in without a provider. Local runs use their own Google and GitHub
registrations with localhost callbacks; tests sign in through a fake provider that exists only in
test code.

**Consequences:**
- ADR 0019 puts identity in its own hexagon, and ADR 0020 gives the developer page its own server.
- User-flow decision 12 is settled: the bootstrap command grants only the first developer.
- The in-memory sample and its `dev` settings (ADR 0011) stay until a database exists (STU-28), so
  building sign-in follows STU-28. Until then the seeded `dev.agent_token` doesn't follow the
  agent-token rules: it's known debt (ADR 0015), and its comments name STU-85, which removes it.
- The domain and CDN in front of the Function URLs are STU-37's. Rate limits, concurrency caps and
  billing alarms are STU-32's.
- Content made on the developer page lives in the database, which STU-27 and STU-28 decide.
- Deleting an account (E13) spans both hexagons and is designed with E13.
