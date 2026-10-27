# Authoring and tuning content

Everything a player and her coach see comes from `content/`. The server loads it at startup;
restart after editing. `cargo test` loads the real content, so a broken file fails the tests
before it reaches a session.

## Tuning the coach

Start with [`content/coach.toml`](../content/coach.toml): reply length, questions per reply,
how often prompt tips appear, the player profile, project details. Values are used everywhere as
`{{section.key}}`. An unknown placeholder fails loading, so typos can't leak into a briefing.

The coach's behaviour lives in [`content/core.md`](../content/core.md); it is shared by every
checkpoint. Checkpoint-specific coaching lives in each `brief.md`.

A practical way to tune: play a checkpoint yourself in Claude Code, then adjust `coach.toml`
first and `core.md` second.

## A checkpoint

```
content/levels/03-preparing/
  level.toml          metadata + criteria (checks never leave the server)
  brief.md            for the coach: why, role, flow, scope, traps, what the handoff covers
  page.md             for the player, on the website: short, with weak vs better prompts
  guides/1-hint.md    escalating help: a hint, then an example, then a walkthrough
  guides/2-....md
```

Directory names are stable ids; numbers in `level.toml` must run 0, 1, 2... without gaps.

### Criteria

```toml
[[criteria]]
id = "round-trip"                    # stable; stored with every submission
title = "Save and load one registration"
ask = """What the coach must collect and how. Phrase it for the coach."""
evidence = "output"                  # output = raw terminal output, text = her words
[criteria.check]
kind = "key_values"
expected = { name = "Noemí Albornoz", email = "noemi.albornoz@example.com" }
```

Guidelines:

- **One thing per criterion.** The coach sees them one at a time, in order.
- **`ask` says what to submit, not what passes.** The check is the hidden part.
- **Rubrics:** say what must be present, what is generous, and when to fail. Add
  `fallback_keywords` groups so the offline grader behaves sensibly.
- **Code checks:** prefer `key_values` on numbers her program computes from the sample data.

### Sample data and hidden answers

Shared files (`content/shared/`) are served to the coach by name. If you change the CSVs or the
rules, recompute the answers and update the `expected` tables:

```bash
python3 tools/reference_answers.py
```

`expected_values_match_reference` fails until they agree.
