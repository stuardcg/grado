# Roadmap

What v1 deliberately leaves out, and where each piece plugs in.

## Before real players

- **Real sign-in.** Replace the development sign-in with an identity provider (WorkOS AuthKit or
  Clerk). Add a route that completes the provider's callback and calls
  `http::auth::start_session`; add a `GRADO_AUTH_MODE` variant in `config.rs`.
- **Grader key.** Set `ANTHROPIC_API_KEY`. Without it, rubric checks use keyword matching.
- **Play it yourself.** Run each checkpoint in Claude Code and tune `content/coach.toml` and
  `core.md` against how it feels.

## Hosting

- **Database.** Implement `Store` for Postgres (`sqlx` with the `postgres` feature) with its own
  migrations under `crates/grado-store/migrations/postgres/`, and pick the implementation from
  `GRADO_DATABASE_URL` in `build_state`. Neon works well for this size.
- **Vercel.** The app is one axum `Router` (`grado_server::router`), and the MCP transport is
  stateless, so it can run behind a serverless adapter. Check the current state of Vercel's Rust
  runtime first; if it doesn't fit, run the same binary on Fly.io or Railway and keep `web/` on
  Vercel. Set `GRADO_PUBLIC_URL` and `GRADO_ALLOWED_HOSTS` to the public host.
- **Content on serverless.** `content/` and `web/` are read from disk at startup. Bundle them
  with the function, or embed them at compile time (`include_dir`).

## Later

- **MCP OAuth.** Replace personal access tokens with the MCP authorization flow once the identity
  provider supports it; `mcp::require_bearer` is the only place that changes.
- **Rate limiting** on `submit_evidence` (the attempt log already records every call).
- **More journeys.** Content is data: a second route is a second content directory.
