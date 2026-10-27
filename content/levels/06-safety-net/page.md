You're about to climb the ridge: next checkpoint you'll reorganize all your code. Before that, you'll
build a **safety net**: tests that check the event rules automatically, in a second.

### The habit: define "done" with examples

"Riders get the right category" is vague. "Someone born on 14 March 2009 is Elite" is precise: it's a
test, a spec and a good prompt at the same time.

| Weak | Better |
|---|---|
| "Add tests." | "Write a test for: rider born 2009-03-14 is Elite, born 2009-03-15 is Junior." |
| "Test duplicates." | "Write a test: `Ana@Example.com ` and `ana@example.com` count as the same rider; the first is kept." |

### The best minute of this checkpoint

Break a rule on purpose and watch a test catch it.
