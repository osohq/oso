class DataFilter:
    """An object representing an abstract query over a particular data type"""

    def __init__(self, model, relations, conditions, types):
        self.model = model
        self.relations = relations
        self.conditions = conditions
        self.types = types

    @classmethod
    def parse(cls, polar, blob):
        types = polar.host.types
        model = types[blob["root"]].cls
        relations = [Relation.parse(polar, *rel) for rel in blob["relations"]]
        conditions = [
            [Condition.parse(polar, *conj) for conj in disj]
            for disj in blob["conditions"]
        ]

        return cls(model=model, relations=relations, conditions=conditions, types=types)


class Projection:
    """
    An object representing a named property (`field`) of a particular data type (`source`).
    `field` may be `None`, which user code must translate to a field (usually the primary key
    column in a database) that uniquely identifies the record.
    """

    def __init__(self, source, field):
        self.source = source
        self.field = field


class Relation:
    """An object representing a named relation between two data types"""

    def __init__(self, left, name, right):
        self.left = left
        self.name = name
        self.right = right

    @classmethod
    def parse(cls, polar, left, name, right):
        left = polar.host.types[left].cls
        right = polar.host.types[right].cls
        return cls(left=left, name=name, right=right)


class Condition:
    """
    An object representing a WHERE condition on a query.

    `cmp` is an equality or inequality operator.

    `left` and `right` may be Projections or literal data.
    """

    def __init__(self, left, cmp, right):
        self.left = left
        self.cmp = cmp
        self.right = right

    @classmethod
    def parse(cls, polar, left, cmp, right):
        left = cls.parse_side(polar, left)
        right = cls.parse_side(polar, right)
        return cls(left=left, cmp=cmp, right=right)

    @staticmethod
    def parse_side(polar, side):
        key = next(iter(side.keys()))
        val = side[key]
        if key == "Field":
            source = polar.host.types[val[0]].cls
            field = val[1]
            return Projection(source=source, field=field)
        elif key == "Immediate":
            return polar.host.to_python(
                {"value": {next(iter(val.keys())): next(iter(val.values()))}}
            )
        else:
            raise ValueError(key)
