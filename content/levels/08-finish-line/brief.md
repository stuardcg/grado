## Why this checkpoint exists

The snowy peak. {{project.organizer}} writes:

> The forest trail is narrow, so we have to cap **Elite at 8 riders**. Extra Elite riders go on a
> **waitlist**, in the order they registered. The start list should show the waitlist separately, and
> I need the waitlist count in the summary.

She handles it twice: in her real app, planned and tested, and in the speedrun app from checkpoint 0.
The comparison is the payoff of the whole journey. Don't tell her what she'll feel; let her find out.

## Your role here

1. Deliver {{project.organizer}}'s message (quote it as above).
2. **Planning first.** Ask her to update `plan.md` with a short change plan, using her existing plan and
   the handoff as context. That's today's habit: the plan is the context, so she doesn't re-explain the
   app. Coach gaps; she writes.
3. **Real app.** Implement it step by step from her plan: capacity as part of the category data, tests
   first if she's willing (great moment to show it), then the rule, the summary line, the PDF section.
   Run tests after each step.
4. **Speedrun app.** Switch to `speedrun/`. Time-box 20 minutes. Behave like a generic assistant would
   with a one-shot request there: she asks, you do. Don't secretly restructure the speedrun app to make
   it easier, and don't sabotage it either. If it gets messy, that's the observation.
5. Ask for her comparison (last check). Listen; don't lead.

## Rules for the waitlist

Rejected rows and duplicates are decided exactly as before. Among accepted Elite riders, in file order,
the first 8 are in; the rest go on the waitlist. Other categories have no cap. Capacity lives with the
category data (e.g. an optional `capacity` field), not in an if.

## Scope

- **In:** the capacity change in both apps, tests for it, the PDF waitlist section.
- **Out:** other new features. This is the last checkpoint; parked ideas can be listed in the handoff as
  "next adventures".

## Common traps

- Hardcoding `8` in the rule instead of on the category data.
- Waitlisting before removing rejects/duplicates. Order matters.
- Spending more than 20 minutes on the speedrun app. Stop; the struggle is already the data point.

## What your closing handoff note should cover

What changed, the final test count, her comparison in one line, and her parked ideas as a list of
"next adventures". This handoff is her summit certificate; make it something she'd want to keep.
