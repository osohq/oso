class DataFilter:
    def __init__(self, model, relations, conditions, types):
        self.model = model
        self.relations = relations
        self.conditions = conditions
        self.types = types

    def parse(polar, blob):
        types = polar.host.types
        model = types[blob["root"]].cls
        relations = [Relation.parse(polar, *rel) for rel in blob["relations"]]
        conditions = [
            [Condition.parse(polar, *conj) for conj in disj]
            for disj in blob["conditions"]
        ]

        return DataFilter(
            model=model, relations=relations, conditions=conditions, types=types
        )


class Projection:
    def __init__(self, source, field):
        self.source = source
        self.field = field


class Relation:
    def __init__(self, left, name, right):
        self.left = left
        self.name = name
        self.right = right

    def parse(polar, left, name, right):
        left = polar.host.types[left].cls
        right = polar.host.types[right].cls
        return Relation(left=left, name=name, right=right)


class Condition:
    def __init__(self, left, cmp, right):
        self.left = left
        self.cmp = cmp
        self.right = right

    def parse(polar, left, cmp, right):
        left = Condition.parse_side(polar, left)
        right = Condition.parse_side(polar, right)
        return Condition(left=left, cmp=cmp, right=right)

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
