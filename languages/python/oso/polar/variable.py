class Variable(str):
    """An unbound variable type, can be used to query the KB for information"""

    def __repr__(self):
        return f"Variable({super().__repr__()})"

    def __str__(self):
        return repr(self)

    def __eq__(self, other):
        return super().__eq__(other)

    def __hash__(self):
        return super().__hash__()
