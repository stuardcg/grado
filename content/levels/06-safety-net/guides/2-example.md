# Example: a test that reads like a rule

```python
def test_rider_turning_18_on_race_day_is_elite():
    assert category_for(birth_date="2009-03-14") == "Elite"
```

The name says the rule. The body is one example. That's the whole trick.
