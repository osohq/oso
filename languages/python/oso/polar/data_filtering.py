from typing import Any, Optional
from dataclasses import dataclass
from collections import defaultdict
from functools import reduce


# Used so we know what fetchers to call and how to match up constraints.
@dataclass
class Relation:
    """An object representing a relation between two types registered with Oso."""

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
            if isinstance(v, Relation):
                field_types[k] = {
                    "Relation": {
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


binary_predicates = {
    "Eq": lambda a, b: a == b,
    "Neq": lambda a, b: a != b,
    "In": lambda a, b: a in b,
    "Nin": lambda a, b: a not in b,
    "Contains": lambda a, b: b in a,
}


@dataclass
class Filter:
    """An object representing a predicate on a type registered with Oso."""

    kind: str
    field: str
    value: Any

    def __post_init__(self):
        if isinstance(self.value, Field):
            self.other_val = lambda x: getattr(x, self.value.field)
        else:
            self.other_val = lambda x: self.value

        if self.field is None:
            self.my_val = lambda x: x
        elif type(self.field) is list:
            self.my_val = lambda x: [_getattr(x, f) for f in self.field]
        elif type(self.field) is str:
            self.my_val = lambda x: getattr(x, self.field)

    def check(self, item):
        return binary_predicates[self.kind](self.my_val(item), self.other_val(item))

    def ground(self, polar, results):
        if isinstance(self.value, Ref):
            ref = self.value
            self.value = results[ref.result_id]
            if ref.field is not None:
                self.value = [getattr(v, ref.field) for v in self.value]


def _getattr(x, attr):
    return x if attr is None else getattr(x, attr)


def ground_filters(results, filters):
    def is_field_ref(fil):
        return isinstance(fil.value, Ref) and fil.value.result_id is not None

    refs, rest = partition(filters, is_field_ref)
    yrefs, nrefs = partition(refs, lambda r: r.kind == "In" or r.kind == "Eq")
    for refs, kind in [(yrefs, "In"), (nrefs, "Nin")]:
        for rid, fils in group_by(refs, lambda f: f.value.result_id).items():
            if len(fils) > 1:
                value = [
                    [_getattr(r, f.value.field) for f in fils] for r in results[rid]
                ]
                field = [f.field for f in fils]
                rest.append(Filter(value=value, kind=kind, field=field))
            else:
                fil = fils[0]
                rest.append(
                    Filter(
                        field=fil.field,
                        kind=kind,
                        value=[_getattr(r, fil.value.field) for r in results[rid]],
                    )
                )
    return rest


def partition(coll, pred):
    def step(m, x):
        (m[0] if pred(x) else m[1]).append(x)
        return m

    return reduce(step, coll, ([], []))


def group_by(coll, kfn):
    def step(m, x):
        m[kfn(x)].append(x)
        return m

    return reduce(step, coll, defaultdict(list))


def parse_constraint(polar, constraint):
    kind = constraint["kind"]
    assert kind in ["Eq", "Neq", "In", "Nin", "Contains"]
    field = constraint["field"]
    value = constraint["value"]

    value_kind = next(iter(value))
    value = value[value_kind]

    if value_kind == "Term":
        value = polar.host.to_python(value)
    elif value_kind == "Ref":
        child_field = value["field"]
        result_id = value["result_id"]
        value = Ref(field=child_field, result_id=result_id)
    elif value_kind == "Field":
        value = Field(field=value)
    else:
        assert False, "Unknown value kind"

    return Filter(kind=kind, field=field, value=value)


# @NOTE(Steve): This is just operating on the json. Could still have a step to parse this into a python data structure
# first. Probably more important later when make implementing a resolver nice.
def builtin_filter_plan_resolver(polar, filter_plan):
    result_sets = filter_plan["result_sets"]
    queries = []
    result_type = None
    for rs in result_sets:
        set_query = None
        set_results = {}

        requests = rs["requests"]
        resolve_order = rs["resolve_order"]
        result_id = rs["result_id"]

        for i in resolve_order:
            req = requests[str(i)]  # thanks JSON
            class_name = req["class_tag"]
            constraints = req["constraints"]

            constraints = [parse_constraint(polar, c) for c in constraints]
            constraints = ground_filters(set_results, constraints)
            # Substitute in results from previous requests.
            cls_type = polar.host.types[class_name]
            query = cls_type.build_query(constraints)
            if i != result_id:
                set_results[i] = cls_type.exec_query(query)
            else:
                set_query = query
                result_type = cls_type

        queries.append(set_query)

    if len(queries) == 0:
        return None

    result_query = queries[0]
    for q in queries[1:]:
        result_query = result_type.combine_query(result_query, q)

    return result_query


def filter_data(polar, filter_plan, filter_plan_resolver=builtin_filter_plan_resolver):
    return filter_plan_resolver(polar, filter_plan)
