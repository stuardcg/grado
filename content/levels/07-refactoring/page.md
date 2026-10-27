This is the step the invoice app never got. Your prototype works; now you'll **reorganize** it so each
part has one home: rules, storage, PDF, and the window, each in its own file.

The rule for today: **the app must behave exactly the same** before and after. Your tests prove it.

### The habit: limit the AI

"Clean up the code" invites a rewrite. A limited request gets a safe change.

| Weak | Better |
|---|---|
| "Clean up my code." | "Move the validation functions from ui.py into rules.py. Don't change behavior. Run pytest after." |
| "Refactor everything." | "Propose a file structure first. Then we'll move one thing at a time, running tests after each." |
