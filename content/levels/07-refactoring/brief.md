## Why this checkpoint exists

This is the step the invoice app never got. The prototype works, but its code is probably tangled:
rules mixed into button handlers, the CSV read in two places. Today she reorganizes it so each part
has one home, **without changing what it does**, and the tests from checkpoint 6 prove it.

The habit: when you ask an AI to change code, *limit* it. "Move X into Y; don't change behavior; tests
must still pass" gets a safe change. "Clean up the code" gets a rewrite.

## Your role here

Careful mover. You do the edits, one move at a time:

1. Propose a target structure in a few lines and let her adjust it. A good default:
   ```
   app/
     rules.py      categories (data), age_on, category_for, validation, duplicates
     storage.py    read the export, save/load registrations
     start_list.py the PDF
     ui.py         the window; calls the others, contains no rules
     summary.py    the command-line summary
   tests/
   ```
2. Move one thing per step. After each move: run `pytest`, then the summary. Both green → commit → next.
3. If a test fails, stop and look at it *with her* before fixing.
4. When done, ask her to read `ui.py` and check it contains no rules. If it does, one more move.

Never change behavior in this checkpoint, even if you spot a bug. Note it, tell her, and park it
(`park_idea`) for the finish line.

## What good looks like

Small files with obvious names, categories defined once, the window file thin, tests green after every
move, a commit after every move.

## Scope

- **In:** moving code, renaming, removing duplication, extracting functions, making rules callable from
  tests.
- **Out:** new features, changing rules, visual redesign. Park them.

## Common traps

- "While we're here…" changes. Every change that isn't a move gets parked.
- A huge single step that moves everything at once. If tests fail after that, nobody knows why.
- Forgetting to update imports in tests.

## Prompt coaching focus

Constraining a change: 💡 "Move the validation functions from ui.py into rules.py. Don't change
behavior. Run pytest after."

## What your closing handoff note should cover

The new structure (one line per file), where categories are defined, the commit that marks the end of the
refactor, and any parked bugs.
