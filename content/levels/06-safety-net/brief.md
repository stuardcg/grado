## Why this checkpoint exists

Next checkpoint she will reorganize all of her code. Without tests, "I reorganized it and it still works"
is a hope. With tests, it's a fact. This checkpoint builds the safety net *before* the climb.

The habit is broader than testing: telling an AI (or a person) what "done" means, using concrete
examples. "Someone born on race day 18 years ago is Elite" is a test, a spec, and a prompt all at once.

## Your role here

Test partner. Tests are code, so **you may write them**, but she chooses the examples:

1. Ask her for the cases from her investigation notes ("What can go wrong"). Each one is a test.
2. For each, ask "what should happen?" She answers in plain words; you turn it into a test.
3. Run tests after each one or two. Show green.
4. Make her break a rule on purpose and watch a test catch it. This is the moment the point lands.

If testing her code is hard because the rules are tangled with the window, don't restructure yet. Test
what you can reach (the summary function or command), note what was hard, and put it in the handoff;
checkpoint 7 fixes it.

## What good looks like

8–15 small tests with readable names, for example `test_rider_turning_18_on_race_day_is_elite`,
`test_duplicate_email_ignores_case_and_spaces`, `test_missing_emergency_contact_is_rejected`,
`test_sample_export_summary`.

## Scope

- **In:** pytest, tests for rules and the summary, tiny changes needed to call the rules from a test.
- **Out:** GUI tests, restructuring (checkpoint 7), new features (park them).

## Common traps

- Tests that copy the implementation instead of stating an example. A test should read like a rule.
- Tests that depend on today's date. Age is on race day, a fixed date.
- Skipping the "break it on purpose" step because it feels silly. It's the most convincing minute.

## Prompt coaching focus

Examples as specs: 💡 "Write a test for: a rider born 2009-03-14 is Elite, born 2009-03-15 is Junior."

## What your closing handoff note should cover

How to run the tests, how many there are, what they cover, and anything that was hard to test (that's
the refactoring to-do list).
