# Example Valley MTB 2027: notes from the organizer

> This file is for the coach. In Investigation, the coach plays the organizer and only
> answers what she asks. From Learning on, she may read it directly.

**Event:** Example Valley MTB 2027, a mountain bike race. **Race day:** Sunday 14 March 2027.
**Organizer:** Alex. Small team of volunteers. Registrations arrive by email and from a
web form the local club runs; Alex gets them as a spreadsheet export (CSV) and sometimes
types late ones by hand.

## What Alex needs

- Keep every registration in one place on her laptop. No internet needed on race day.
- Know how many riders are in each category, so she can plan medals, water and marshals.
- Print a start list on race day, grouped by category, with each rider's name and
  emergency contact. Marshals carry it on a clipboard.
- Catch bad data before race day. Last year two riders were in the wrong category and a
  kid's form had no emergency contact. That cannot happen again.

## A registration

| Field | Required | Notes |
|---|---|---|
| Registration ID | yes | From the club's form, e.g. `R014`. Hand-typed ones may not have one. |
| Full name | yes | |
| Email | yes | Used to tell riders apart. One registration per email. |
| Birth date | yes | `YYYY-MM-DD`. |
| Phone | no | |
| Emergency contact | yes | Name and phone, free text. Non-negotiable. |
| T-shirt size | no | XS, S, M, L, XL. |

## Categories

The category comes from the rider's **age on race day**, not on the day they registered.

| Category | Age on race day | Distance |
|---|---|---|
| Kids | 8 to 12 | 10 km |
| Junior | 13 to 17 | 30 km |
| Elite | 18 to 39 | 60 km |
| Master | 40 and over | 60 km |

Riders younger than 8 cannot take part. Ages are inclusive: a rider who turns 18 on race
day races Elite.

## Rules for the spreadsheet export

- Go through rows in order.
- **Reject** a row if the name is empty, the email is not a valid address, the birth date
  is missing or not a real date, the emergency contact is empty, or the rider is under 8.
- A row that is fine but uses an email already seen (ignoring capital letters and spaces
  around it) is a **duplicate**: keep the first one, skip the later one.
- Rejected rows never count as "already seen".

## Things Alex mentions if asked

- She would love to add a "Family 10k" category next year.
- She is not sure yet whether Elite needs a cap; the forest trail is narrow.
- Payments are handled by the club. Not her problem, not the app's problem.
- She prints on A4.
