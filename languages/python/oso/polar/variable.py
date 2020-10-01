class Variable(str):
    """An unbound variable type, can be used to query the KB for information"""

    def __repr__(self):
        return f"Variable({super().__repr__()})"

    def __str__(self):
        return repr(self)

    def __eq__(self, other):
        return isinstance(other, type(self)) and super().__eq__(other)
