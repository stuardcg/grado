# Grado

A route of checkpoints that teaches how to build software *with* an AI: investigate, learn,
prepare, plan, prototype, test, refactor, and handle a change request. Each checkpoint
teaches one habit.

The player works in **Claude Code**. A coaching agent connects to this server over MCP to get
the current checkpoint's briefing and to submit evidence. The server holds the acceptance
criteria and judges the evidence; the agent only ever sees the current check. When every check
passes, the agent gets a one-time **stamp**. The player enters it on the website to unlock the
next checkpoint.

```
Player ──▶ Claude Code (coach) ──MCP──▶ Grado server ◀──HTTP── Website (player, game master)
                                           │
                                           ├─ content/  briefings, criteria, guides (git)
                                           └─ SQLite    players, progress, evidence
```

The sample journey builds a registration desktop app for a mountain bike race:
9 checkpoints from *Meadow* to *Snowy peak*. See [`content/core.md`](content/core.md) for the
route.

## Run it locally

Requires Rust (stable, 2024 edition) and, for one content test, Python 3.

```bash
cargo run -p grado-server
```

Open http://localhost:8787, sign in with any name and email (development sign-in), then:

1. **Connect** → name the computer → copy the `claude mcp add ...` command and run it.
2. In a project folder, start `claude` and type `/mcp__grado__checkpoint`.
3. When the coach hands over a stamp, enter it on the checkpoint page.

To see the game master view, start the server with your email as admin:

```bash
GRADO_ADMIN_EMAILS=you@example.com cargo run -p grado-server
```

Every setting is listed in [`.env.example`](.env.example). Without `ANTHROPIC_API_KEY`, text
checks are graded by an offline keyword grader. That's fine for trying the route, but too
lenient for real play.

## Tests

```bash
cargo test --workspace
```

Covers the evidence checks, storage, the full checkpoint flow against the real content, and
that the hidden expected values match [`tools/reference_answers.py`](tools/reference_answers.py).

## Layout

| Path | What lives there |
|---|---|
| `content/` | Everything a checkpoint *is*: coach, briefings, criteria, guides, sample data. Edit here to tune the game. |
| `crates/grado-domain` | Loads content; decides what is visible; judges evidence. No I/O besides reading content. |
| `crates/grado-store` | Persistence behind the `Store` trait. SQLite today. |
| `crates/grado-server` | Axum app: JSON API, sign-in, MCP endpoint, static website, Claude grader. |
| `web/` | The website. Plain HTML/CSS/JS, no build step. |
| `tools/` | Reference implementation of the event rules, used to compute the hidden answers. |
| `docs/` | [Architecture](docs/architecture.md), [authoring content](docs/authoring.md), [roadmap](docs/roadmap.md). |
