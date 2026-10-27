# Walkthrough: the script, piece by piece

1. Read the file into a list of rows (`csv.DictReader`).
2. Write `age_on(birth_date, race_day)`.
3. Write `category_for(age)` that looks through the `CATEGORIES` list.
4. Make a dict of counts, starting at 0 for every category.
5. For each rider: age → category → add 1.
6. Print `Category: count` for each category.

Build and run after each step. Six small wins.
