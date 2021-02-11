---
submitted_by: submitted_by
postfixId: "_id"

expenseClass: |
    @polar_class
    class Expense:
        ...

userClass: |
    @polar_class
    class User:
        def __init__(self, name, location: str = None):
            ...
---
