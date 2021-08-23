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
class Constraint:
    kind: str  # ["Eq", "In", "Contains"]
    field: str
    value: Any

    def __post_init__(self):
        if isinstance(self.value, Field):
            self.getter = lambda x: getattr(x, self.value.field)
        else:
            self.getter = lambda x: self.value

        if self.field is None:
            self.iget = lambda x: x
        else:
            self.iget = lambda x: getattr(x, self.field)

        if self.kind == "Eq":
            self.checker = lambda a, b: a == b
        elif self.kind == "In":
            self.checker = lambda a, b: a in b
        elif self.kind == "Contains":
            self.checker = lambda a, b: b in a
        else:
            assert False, "unknown constraint kind"

    def check(self, item):
        return self.checker(self.iget(item), self.getter(item))


def parse_constraint(polar, constraint):
    kind = constraint["kind"]
    assert kind in ["Eq", "In", "Contains"]
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

    return Constraint(kind=kind, field=field, value=value)


def ground_constraints(polar, results, filter_plan, constraints):
    for constraint in constraints:
        if isinstance(constraint.value, Ref):
            ref = constraint.value
            constraint.value = results[ref.result_id]
            if ref.field is not None:
                constraint.value = [getattr(v, ref.field) for v in constraint.value]


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
            constraints = req["constraints"]

            constraints = [parse_constraint(polar, c) for c in constraints]

            # Substitute in results from previous requests.
            ground_constraints(polar, set_results, filter_plan, constraints)
            fetcher = polar.host.types[class_name].fetcher
            set_results[i] = fetcher(constraints)

        results.extend(set_results[result_id])

    # NOTE(steve): Not the best way to remove duplicates.
    return [i for n, i in enumerate(results) if i not in results[:n]]


def filter_data(polar, filter_plan, filter_plan_resolver=builtin_filter_plan_resolver):
    return filter_plan_resolver(polar, filter_plan)
