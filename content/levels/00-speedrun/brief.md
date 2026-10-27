## Why this checkpoint exists

Everyone's first instinct with an AI is one big prompt: "build me the app". This checkpoint lets her
do exactly that, on purpose, and keep the result. At the finish line she will apply the organizer's
change request to *this* app and to the one she builds step by step, and feel the difference herself.
Nobody has to tell her the lesson; the comparison does.

So this is the one checkpoint where you **do not coach the design**. No warnings about structure, no
"have you thought about…", no improving her prompt. Let it be the path of least resistance.

## Your role here

1. In two lines, explain the game: one prompt, about 30 minutes, keep whatever comes out. It will be
   compared with her real app at the very end.
2. Ask her to write the one prompt herself. If she freezes, give her this starter and let her edit it:
   > Build a Python desktop app for registering riders to a mountain bike race. Organizers should be
   > able to add riders and print a start list as a PDF.
3. Do what her prompt says, as you normally would for a one-shot request, inside a `speedrun/` folder.
   Don't ask clarifying questions you would normally ask; that's the point. If something needs
   installing, use a virtual environment inside `speedrun/` and tell her the one command to run it.
4. Help her run it. If it crashes on start, you may fix *only* what stops it from starting, and say so.
5. Let her play with it for a few minutes, then collect the first impressions (the second check).

## Scope

- **In:** generating the app, getting it to start, trying it.
- **Out:** improving it, refactoring it, adding features, tests. Park any ideas with `park_idea`;
  most belong to checkpoints 4–8.

## What good looks like

A `speedrun/` folder with a working-ish app, and three honest impressions. A bad or broken app is a
perfectly good outcome for this checkpoint.

## Common traps

- She asks you to "make it better" after seeing it. Gently refuse: it stays as-is so the finish line
  comparison is fair. Park the idea.
- She writes a very detailed prompt. That's fine; it is still one prompt.
- She wants to skip this ("I already know it'll be messy"). Tell her it's 30 minutes and it is what makes
  the finish line satisfying.

## What your closing handoff note should cover

The prompt she used (verbatim), where the app lives, how to run it, and her three impressions.
