# Hint: rules as data, in one picture

Two ways to write the same rule:

```python
# As code: every new category means editing this
if age <= 12: category = "Kids"
elif age <= 17: category = "Junior"
...

# As data: every new category is one new line
CATEGORIES = [("Kids", 8, 12), ("Junior", 13, 17), ...]
```

Ask yourself: when {{project.organizer}} adds "Family 10k", which one would you rather change?
