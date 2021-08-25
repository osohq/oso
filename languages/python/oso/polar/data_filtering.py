from typing import Any, Optional
from dataclasses import dataclass

VALID_KINDS = ["parent", "children"]


# Used so we know what fetchers to call and how to match up constraints.
@dataclass
class Relationship:
    kind: str
    other_type: str
    my_field: str
    other_field: str


# @NOTE(Steve): Some of this stuff is very inconsistent right now. Names for fields
# and stuff need cleaning up. Sort of left a mess from when I was figuring this all
# out.
def serialize_types(types, tmap):
    """
    Convert types stored in python to what the core expects.
    """
    polar_types = {}
    for typ in types:
        tag, fields = typ.name, typ.fields
        field_types = {}
        for k, v in fields.items():
            if isinstance(v, Relationship):
                field_types[k] = {
                    "Relationship": {
                        "kind": v.kind,
                        "other_class_tag": v.other_type,
                        "my_field": v.my_field,
                        "other_field": v.other_field,
                    }
                }
            else:
                field_types[k] = {
                    "Base": {
                        "class_tag": tmap[v].name,
                    }
                }
        polar_types[tag] = field_types
    return polar_types


@dataclass
class Field:
    field: str


@dataclass
class Ref:
    field: Optional[str]
    result_id: int


@dataclass
class Loc:
    field: Optional[str]
    result: Optional[int]
    def get(self, item):
        return item if self.field is None else getattr(item, self.field)


@dataclass
class Val:
    term: Any
    def get(self, item):
        return self.term


@dataclass
class Con:
    kind: str
    left: Any
    right: Any

    def Eq(a, b):
        return a == b
    def Neq(a, b):
        return a != b
    def In(a, b):
        return a in b
    def Contains(a, b):
        return b in a

    def check(self, item):
        return getattr(type(self), self.kind)(self.left.get(item), self.right.get(item))


def parse_con(polar, con):
    def parse_val(val):
        kind = next(iter(val))
        val = val[kind]
        if kind == 'Loc':
            return Loc(field=val['field'], result=val['result'])
        elif kind == 'Val':
            return Val(term=polar.host.to_python(val['term']))
        assert False, "Unknown value kind"

    kind = con['kind']
    assert kind in ["Eq", "In", "Neq", "Contains"]
    left = parse_val(con['left'])
    right = parse_val(con['right'])
    return Con(kind=kind, left=left, right=right)


def ground_cons(polar, results, filter_plan, cons):
    def do_side(con, side):
        loc = getattr(con, side)
        if isinstance(loc, Loc):
            if loc.result is not None:
                val = Val(term=results[loc.result])
                setattr(con, side, val)
                if loc.field is not None:
                    val.term = [getattr(v, loc.field) for v in val.term]
    for con in cons:
        do_side(con, 'left')
        do_side(con, 'right')

# @NOTE(Steve): This is just operating on the json. Could still have a step to parse this into a python data structure
# first. Probably more important later when make implementing a resolver nice.


def builtin_filter_plan_resolver(polar, filter_plan):
    result_sets = filter_plan["result_sets"]
    results = []
    for rs in result_sets:
        set_results = {}

        requests = rs["requests"]
        resolve_order = rs["resolve_order"]
        result_id = rs["result_id"]

        for i in resolve_order:
            req = requests[str(i)]  # thanks JSON
            class_name = req["class_tag"]
            constraints = req["cons"]

            constraints = [parse_con(polar, c) for c in constraints]

            # Substitute in results from previous requests.
            ground_cons(polar, set_results, filter_plan, constraints)
            fetcher = polar.host.types[class_name].fetcher
            set_results[i] = fetcher(constraints)

        results.extend(set_results[result_id])

    # NOTE(steve): Not the best way to remove duplicates.
    return [i for n, i in enumerate(results) if i not in results[:n]]


def filter_data(polar, filter_plan, filter_plan_resolver=builtin_filter_plan_resolver):
    return filter_plan_resolver(polar, filter_plan)
