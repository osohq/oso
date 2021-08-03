from typing import Any
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
def serialize_types(types, class_names):
    """
    Convert types stored in python to what the core expects.
    """
    polar_types = {}
    for tag, fields in types.items():
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
                        "class_tag": class_names[v],
                    }
                }
        polar_types[tag] = field_types
    return polar_types

@dataclass
class FetchResult:
    id: str

@dataclass
class Attrib:
    field: str
    of: FetchResult

@dataclass
class Constraint:
    kind: str  # ["Eq", "In"]
    field: str
    value: Any  # Value or list of values.

def parse_constraint(polar, constraint):
    kind = next(iter(constraint))
    assert kind in ["Eq", "In"]
    field = constraint[kind]["field"]
    value = constraint[kind]["value"]
    # @TODO(steve): This is not the best way to distinguish these...
    if 'field' in value:
        fetch_result = FetchResult(id=value['of']['id'])
        attrib = Attrib(field=value['field'], of=fetch_result)
        value = attrib
    elif 'value' in value:
        value = polar.host.to_python(value)
    else:
        assert False, "Unknown constraint kind"
    return Constraint(kind=kind, field=field, value=value)


def ground_constraints(polar, results, filter_plan, constraints):
    for constraint in constraints:
        field = None
        if isinstance(constraint.value, Attrib):
            field = constraint.value.field
            constraint.value = constraint.value.of
        if isinstance(constraint.value, FetchResult):
            constraint.value = results[constraint.value.id]
        if field is not None:
            constraint.value = [getattr(v, field) for v in constraint.value]


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
            req = requests[i]
            class_name = req["class_tag"]
            constraints = req["constraints"]

            constraints = [parse_constraint(polar, c) for c in constraints]

            # Substitute in results from previous requests.
            ground_constraints(polar, set_results, filter_plan, constraints)
            fetcher = polar.host.fetchers[class_name]
            set_results[i] = fetcher(constraints)

        results.extend(set_results[result_id])

    # NOTE(steve): Not the best way to remove duplicates.
    return [i for n, i in enumerate(results) if i not in results[:n]]


def filter_data(polar, filter_plan, filter_plan_resolver=None):
    if filter_plan_resolver is None:
        return builtin_filter_plan_resolver(polar, filter_plan)
    else:
        return filter_plan_resolver(polar, filter_plan)
