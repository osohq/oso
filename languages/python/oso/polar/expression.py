class Expression:
    def __init__(self, operator, args):
        self.operator = operator
        self.args = args

    def __repr__(self):
        return f"Expression({self.operator}, {self.args})"

    def __str__(self):
        return f"Expression({self.operator}, {self.args})"

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.operator == other.operator
            and self.args == other.args
        )


class Pattern:
    def __init__(self, tag, fields):
        self.tag = tag
        self.fields = fields

    def __repr__(self):
        return f"Pattern({self.tag}, {self.fields})"

    def __str__(self):
        return repr(self)

    def __eq__(self, other):
        return (
            isinstance(other, type(self))
            and self.tag == other.tag
            and self.fields == other.fields
        )
