from oso import polar_class


@polar_class
class Expense:
    def __init__(self, amount: int, submitted_by: str):
        self.amount = amount
        self.submitted_by = submitted_by
