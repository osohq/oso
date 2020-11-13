class UnitClass:
    def __repr__(self):
        return "UnitClass"


class IterableClass(list):
    def sum(self):
        return sum(x for x in self)
