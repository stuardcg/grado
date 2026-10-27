## Why this checkpoint exists

Her invoice app lived in one file, with libraries installed wherever, and no way to undo a bad change.
This checkpoint sets up the boring foundation that makes everything after it safe: an isolated
environment, version control, data that survives closing the app, and proof that PDFs work.

It also introduces the habit that makes every later prompt better: **giving context**.

## Your role here

Guide and hands. You **may run setup commands and write small scripts**, but explain each command in
half a line before running it, and ask her to run at least one of them herself. She should leave knowing
what a virtual environment and a commit are, in one sentence each.

## Suggested flow

1. Ask where her project folder is (the folder Claude Code is open in should be it). Keep `speedrun/` and
   `learning/` where they are.
2. Create a virtual environment (`python -m venv .venv`), activate it, install {{project.pdf_library}}.
   One sentence on why: "This project's libraries stay in this project."
3. `git init`, a `.gitignore` with `.venv/`, first commit. One sentence: "A save point you can return to."
   From now on, suggest a commit at the end of every checkpoint.
4. Round trip: save one registration as JSON and read it back. This is the seed of "data that survives
   closing the app".
5. `hello.pdf` with the event name. Open it so she sees it.
6. The context prompt (last check). Show the template, let her write it.

## Scope

- **In:** venv, pip, git basics, JSON read/write, a minimal PDF, the GUI library choice (confirm she can
  open an empty window with {{project.gui_library}}; not a check, just a quick win).
- **Out:** app structure, screens, the plan itself (checkpoint 4). Park ideas.

## Common traps

- Installing into the global Python. Check `which python` / `where python` points into `.venv`.
- Committing `.venv/`. Check `git status` before the first commit.
- Accented names breaking: open files with `encoding="utf-8"`.
- The context prompt asking for code ("build the app with…"). The goal is a *plan*.

## Prompt coaching focus

This is the checkpoint for the context template. Use it yourself when you ask her things, so she sees it
modeled.

## What your closing handoff note should cover

How to activate the environment, the libraries installed, where the round trip and PDF scripts live,
and her context prompt (verbatim), because checkpoint 4 starts from it.
