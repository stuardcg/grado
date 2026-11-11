# 0014. Repository layout and servers

- **Status:** Accepted
- **Date:** 2026-11-11
- **Ticket:** STU-31

The repository is one monorepo with the Cargo workspace at its root. Each transport is served by
its own deployable, a composition-root crate named `<transport>-server`. Its binaries are named
after the runtime they start. Each frontend lives in `web/<name>/`.

| Path | Holds |
|---|---|
| `crates/<name>/` | One crate, with the package named `<name>` (ADR 0004) |
| `crates/<transport>-server/` | One deployable: the composition root for one transport |
| `web/<name>/` | One frontend, self-contained, with its own package manager and lockfile. The first is `web/devtools/`. |
| `docs/` | `rules.md`, `decisions/` and `domain/` |
| `content/` | Proof-of-concept content, until STU-27 decides its place |
| `.github/` | CI and Dependabot (ADR 0013) |
| The root | `Cargo.toml`, `deny.toml`, `committed.toml`, `rustfmt.toml` and `grado.example.toml` |

## Servers

- **One transport per deployable.** Agents connect to `mcp-server`. The app and devtools will use
  `http-server`, which comes with `inbound-http`. The two share no router, function or deploy, so
  each one carries only its own authentication, limits and logs. Lambda bills per request, so
  splitting them costs nothing.
- **A server crate does what ADR 0004 gives `app`.** It builds the adapters, loads its
  configuration (ADR 0011), starts logging (ADR 0012) and starts one runtime per file:
  `src/bin/local.rs`, and later `src/bin/lambda.rs`. Its adapter end-to-end tests are in its
  `tests/` (ADR 0013).
- **Other deployables.** A deployable that isn't a server, such as a CLI, is a composition-root
  crate too, named after what it does.
- **Shared startup code.** Two servers would share configuration loading and log setup. A
  decision record gives that code its home when the second server is added, as ADR 0004's "Where
  a change goes" requires.

**Consequences:**
- `app` is renamed `mcp-server`, and its `server` binary is renamed `local`. Everything ADR 0004,
  0010, 0011, 0012 and 0013 say about `app` now applies to each server crate. The old name also
  collided with "the app", the player's frontend in `docs/domain/`.
- ADR 0004 changes in three places:
  - a new transport is a new `inbound-<transport>` crate plus a new `<transport>-server` crate;
  - a new runtime target is `<transport>-server/src/bin/<runtime>.rs`;
  - a new frontend is `web/<name>/`.
- ADR 0003's single router now means one router per server, each with a `local` and a `lambda`
  entry point.
- A frontend's package manager, CI job and Dependabot entry are chosen in the change that adds
  that frontend.
- ADR 0003's 3-minute Lambda timeout was set for grading, which ADR 0006 dropped. STU-37
  revisits it.
