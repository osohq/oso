class Variable(str):
    """An unbound variable type, can be used to query the KB for information"""

    def __repr__(self):
        return f"Variable({super().__repr__()})"

    def __str__(self):
        return repr(self)
