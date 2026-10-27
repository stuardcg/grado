"""Reference implementation of the event rules, used to compute the hidden
expected values in level.toml files.

Run after changing anything in content/shared/:

    python3 tools/reference_answers.py

and copy the printed values into the matching `expected` tables. A Rust test
(`expected_values_match_reference` in grado-server) fails if they drift.
"""

import csv
import json
import sys
from datetime import date
from pathlib import Path

SHARED = Path(__file__).resolve().parent.parent / "content" / "shared"
EVENT_DAY = date(2027, 3, 14)
CATEGORIES = [  # name, min_age, max_age (inclusive, None = no limit)
    ("Kids", 8, 12),
    ("Junior", 13, 17),
    ("Elite", 18, 39),
    ("Master", 40, None),
]
ELITE_CAPACITY = 8


def age_on(birth: date, day: date) -> int:
    return day.year - birth.year - ((day.month, day.day) < (birth.month, birth.day))


def category_for(age: int):
    for name, low, high in CATEGORIES:
        if age >= low and (high is None or age <= high):
            return name
    return None


def parse_birth(value: str):
    try:
        return date.fromisoformat(value.strip())
    except ValueError:
        return None


def valid_email(value: str) -> bool:
    value = value.strip()
    if value.count("@") != 1:
        return False
    local, domain = value.split("@")
    return bool(local) and "." in domain and not domain.startswith(".") and not domain.endswith(".")


def summarize(rows, elite_capacity=None):
    counts = {name: 0 for name, _, _ in CATEGORIES}
    rejected = duplicates = waitlist = 0
    seen = set()
    for row in rows:
        birth = parse_birth(row["birth_date"])
        category = category_for(age_on(birth, EVENT_DAY)) if birth else None
        if (
            not row["full_name"].strip()
            or not valid_email(row["email"])
            or not row["emergency_contact"].strip()
            or category is None
        ):
            rejected += 1
            continue
        email = row["email"].strip().lower()
        if email in seen:
            duplicates += 1
            continue
        seen.add(email)
        if category == "Elite" and elite_capacity is not None and counts["Elite"] >= elite_capacity:
            waitlist += 1
            continue
        counts[category] += 1
    result = dict(counts)
    result["Rejected"] = rejected
    result["Duplicates"] = duplicates
    if elite_capacity is not None:
        result["Elite waitlist"] = waitlist
    return result


def small_counts(rows):
    counts = {name: 0 for name, _, _ in CATEGORIES}
    for row in rows:
        counts[category_for(age_on(parse_birth(row["birth_date"]), EVENT_DAY))] += 1
    return counts


def read(name):
    with open(SHARED / name, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def main():
    answers = {
        "riders_small.csv (level 2)": small_counts(read("riders_small.csv")),
        "registrations.csv (levels 5-7)": summarize(read("registrations.csv")),
        "registrations.csv with Elite capacity (level 8)": summarize(read("registrations.csv"), ELITE_CAPACITY),
    }
    json.dump(answers, sys.stdout, indent=2)
    print()


if __name__ == "__main__":
    main()
