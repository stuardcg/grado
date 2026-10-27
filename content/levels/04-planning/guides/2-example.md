# Example: a plan's shape

For a different app (a book club tracker), a plan this size is plenty:

```markdown
## Data
- Book: title, author, pages
- Member: name, email
- Meeting: date, book, attendees (list of members)
- Saved in books.json

## Screens
- Main: list of meetings, next one highlighted
- Add meeting: pick a book, pick attendees; error if no book chosen

## Steps
1. Book/Member/Meeting as data, load from a sample JSON, print them.
2. Save and load books.json.
3. Main window showing the meetings list.
4. Add-meeting form with the error message.
```
