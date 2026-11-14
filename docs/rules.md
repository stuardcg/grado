# Rules

Every rule in force, one line each, for building and reviewing. Read the section you're working in; open the decision record a rule cites only when you need its reasons. `0004 #5` means rule 5 of [ADR 0004](decisions/0004-architecture.md). The decision records are the source of truth. A change that accepts or supersedes a decision updates this file in the same pull request.

## Crates

- Put each crate in `crates/<name>/`, with the package named `<name>`. (0004)
- Serve each door from its own deployable, a composition-root crate: `mcp-server` for agents, `http-server` for the game page, `developer-server` for the developer page; name a new door's server after who uses it, and a deployable that isn't a server after what it does. (0014, 0020)
- Keep identity (accounts, identity links, sessions, agent connections, sign-in) in its own hexagon, `identity-domain` and `identity-application`; neither it nor the game's core depends on the other. (0019)
- Put each frontend in `web/<name>/`, with its own package manager; add its CI job and Dependabot entry in the same pull request. The game page is `web/game/`, the developer page `web/developer/`. (0014, 0020)
- Depend only as ADR 0004's crate table allows; add a dependency in the change that first uses it. (0004 #1, #2)
- Never make one adapter crate depend on another. (0004 #3)
- Use `tokio` in `application` only as a dev-dependency, to run tests. (0009)
- Keep `serde` out of `domain`: domain types never cross the wire. (0004 #4)
- Put a change where ADR 0004's "Where a change goes" table says, as ADRs 0014, 0019 and 0020 amend it; if no row fits, write a decision record first. (0004, 0014, 0019, 0020)
- Apply every rule for `domain` and `application` to `identity-domain` and `identity-application`, at their own paths. (0019)

## Domain

- Make every game decision in a `domain` function. (0004 #8)
- Never read the clock or generate randomness or ids in `shared`, `domain` or `application`; ask a port. (0004 #12)
- Never let an id or player-readable text be blank; check once, where the type is built. (0007)
- Name a constructor's error `<Type>Error`, an operation's input `<Verb><Noun>`, its success after the operation, and its error `<Operation>Error`. (0008)
- Give each fallible constructor or operation its own error type, listing only what it can return. (0008)
- Never judge the work: accept the agent's pass or fail per criterion, with no evidence. (0006 #4)
- Refuse to approve a step without a handoff note. (0006 #11)
- Resolve a setting as player over project over game. (0006 #8)

## Use cases

- Write one use case per file: `application/src/use_cases/<verb>_<noun>.rs`. (0004 #5)
- Define exactly `Input`, `Output`, `Error` and one public `run` returning `Result<Output, Error>`. (0004 #6)
- Never call another use case; put shared logic in `domain`. (0004 #7)
- Load through ports, call the domain, save through ports; never compare or count port or domain values. (0004 #8)
- Use no numeric or duration literals other than `0` and `1` in `application`. (0004 #9)
- Take the caller's id in `Input` (`PlayerId` in the game, `AccountId` in identity), never a token or session, unless the use case authenticates or ends a session. (0004 #24, 0019)
- Keep `application/src/` to `lib.rs`, `ports/`, `use_cases/` and `testkit/`. (0004 #21)
- Give each use case one method on its core's handle (`Grado` in `application/src/lib.rs`, `Identity` in `identity-application/src/lib.rs`) that only forwards to its `run`. (0010, 0019)

## Errors

- Give `Error` variants data (ids, counts), never sentences. (0004 #13)
- Write caller-facing wording only in an inbound adapter, in one exhaustive `match` over a use case's `Error`. (0004 #13)

## Ports and tests

- Define each port as a trait in `application/src/ports/<concern>.rs`, one concern per file, using only `domain` and `application` types. (0004 #10)
- Write ports as async traits with `async-trait`; use cases take them as `&dyn Port`. (0009)
- Give each port one suite and one fake in `application/src/testkit/<concern>.rs`; run the suite against the fake and every `outbound-*` implementation. (0004 #11)
- Refuse a save when the state changed since it was loaded, and test that refusal in the port's suite. (0004 #25)
- Write use-case tests in the use case's file, on testkit fakes, never on an adapter. (0004 "Tests")
- Turn on the `testkit` feature only in tests: `application`'s own, and `outbound-*` `[dev-dependencies]`. (0004 #23)

## Adapters

- Call use cases from inbound adapters only through `Grado` and `Identity`; never import either core's ports. (0004 #14, 0010, 0019)
- Expose use cases in one file per inbound crate: `inbound-<name>/src/bindings.rs`. (0004 #15)
- Write bindings by hand; each owns its names, request and response shapes, and wording. No registry shared by inbound crates. (0017, 0020)
- Authenticate once per request in each inbound crate, before any binding runs, by calling identity's authentication use case for that door, and hand the bindings the returned `AccountId`, turned into a `PlayerId` with the same value for the game's calls (for MCP, `authenticate_agent_connection` in `call_tool`). (0004 #24, 0017, 0019)
- Bind developer use cases only in `inbound-developer`; never in `inbound-http` or `inbound-mcp`. (0020)
- Decide who may do what in `domain`, through the use case, never in a binding. (0017)
- Only implement ports in outbound adapters; branch only on the technology (encoding, errors, retries, pagination). (0004 #16)
- Keep every numeric or duration literal in an adapter in a named `const`. (0004 #9)
- Only build adapters, load configuration and start a runtime in a server crate: one file per runtime in its `src/bin/` (`local.rs`, `lambda.rs`), configuration in its `src/config.rs`. (0004 #17, 0011, 0014)
- Choose adapters only in the composition root, a server crate's `src/bin/<runtime>.rs`: build the outbound adapters, build `Grado` and `Identity` from them, and hand both to the inbound adapters. (0010, 0014, 0019)
- Give each server one router and two thin entry points, `lambda.rs` and `local.rs`; no API Gateway. (0003, 0014)

## Identity and access

- Sign people in only through a provider app Grado registered (GitHub, Google); never with a password, an email list or a shortcut. (0018)
- Key an identity on `(provider, subject)`, never on email; keep the provider's id and a display name, and none of its tokens. (0018)
- Link a provider only while signed in; refuse an identity another account owns, and refuse unlinking the last one. (0018)
- Check `state`, PKCE, the nonce and the ID token's issuer, audience and expiry; give each provider its own exact callback; return after sign-in only to a path on the door's own origin. (0018)
- Keep a session as a 256-bit random value in a `__Host-` cookie (`HttpOnly`, `Secure`, `SameSite=Lax`), store only its SHA-256 hash, and issue a new one at every sign-in. (0018)
- Record the door that issued each session and refuse it at any other: the game page's lasts 7 days idle and 30 at most, the developer page's 1 day idle and 7 at most. (0018)
- Write a session's or agent token's "last used" at most once an hour, as a touch with no version check. (0018)
- Require the door's own exact `Origin` on every request that changes something; change nothing on a GET. (0018)
- Send a Content-Security-Policy without inline scripts from every page; treat names from providers as untrusted text. (0018)
- Give an agent token a `grado_` prefix and 256 random bits; show it once, store only its SHA-256 hash, expire it after 90 days without use, and refuse a revoked one at once. (0018)
- Check a developer use case's caller role on every call, through a game port and a `domain` function; give a player use case only the caller's `PlayerId`. (0018)
- Grant `developer` only through the bootstrap command's `grant_first_developer`, refused once a developer exists, or on the developer page; never remove the last developer. (0018)
- Log every developer-page call with the developer, the target player and the use case. (0018)
- Keep sign-in fakes in test code only; no binary signs in without a provider. (0018)

## Configuration

- Load configuration only in a server's `src/config.rs`, once at startup: `grado.toml`, then `GRADO_*` environment variables; never read the environment anywhere else. (0011, 0014)
- Give every setting a field in `Config`, one read in `config.rs` and an entry in `grado.example.toml`; never a default in code. (0011)
- Refuse to start on a missing, invalid or unknown setting, listing every problem and never a value. (0011)
- Hand each adapter only its part of `Config`. (0011)

## Logging

- Log only in adapters and server crates, with `tracing`; never in `domain` or `application`. (0012, 0014)
- Log an unexpected failure once, in the inbound adapter where it's mapped to the transport's error, with its cause; outbound adapters return failures without logging them. (0012)
- Give `PortError` the failure's cause; never drop it with `|_|`. (0012)
- Open each inbound adapter's span in one place, never in each binding: the request id, the call's name and, once known, the player id. (0012)
- Never log a token, player-written text or a request body. (0012)

## Shared code

- Give code used in two or more modules the one home ADR 0004's "Shared code" table gives it. (0004 #18)
- Keep game vocabulary out of `shared`. (0004 #19, #22)
- Never name a module or file `util`, `utils`, `helper`, `helpers`, `common`, `misc` or `shared` outside `shared`. (0004 #20)

## Testing

- Test `shared` and `domain` in the module; use `proptest` for an invariant that must hold over any sequence of operations. (0013)
- Test an inbound adapter end to end in its server's `tests/` (`mcp-server/tests/`), through the real binary: every tool, the authentication refusal and every arm of each `Error` `match`. (0013, 0014)
- Run tests with `cargo test`; add no other runner. (0013)

## Exceptions and debt

- Never break a rule; change it first with a decision record. (0015)
- Mark temporary code with a comment naming the ticket that replaces it (`until STU-27 …`); that ticket's change removes the comment. (0015)

## Commits and pull requests

- Write Conventional Commits; the scope is the crate's name when a change stays in one crate. (0005)
- Before every commit, run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. (AGENTS.md)
- Change `main` only through a pull request that passes CI: fmt, clippy, tests, `cargo deny check` and `committed`. (0013)
- Merge pull requests by rebase; never put a merge commit in one. (0013)
- Add a dependency only under a licence `deny.toml` allows, and ignore an advisory only with its id and a reason. (0013)
- Close a decision ticket only once its record is committed. (0001)

## Licence

- Keep every crate on the workspace licence, PolyForm Noncommercial 1.0.0 (`LICENSE.md`). (0016)
- Accept no outside contributions; pull requests and issues are open to collaborators only. (0016)
