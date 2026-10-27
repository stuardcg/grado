Time to set up camp properly. By the end you'll have a real project:

- its **own environment**, so libraries don't leak between projects
- **version control** (git), so every step has a save point you can return to
- a registration that's **saved and loaded back**, the seed of data that survives closing the app
- a first **PDF**, so you know printing works

### The habit: give context

An AI only knows what you tell it. Four lines of context save ten messages of back-and-forth:

```
Goal: …
What I have: …
What's wrong / constraints: …
What I want back: …
```

| Weak | Better |
|---|---|
| "Make the PDF." | "Goal: a test PDF. What I have: fpdf2 installed in .venv. Constraint: A4. What I want back: a script under 20 lines." |
