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
class Ref:
    result_id: Optional[str]
    field: str


@dataclass
class Constraint:
    kind: str  # ["Eq", "In", "Contains"]
    field: str
    value: Any

    def to_predicate(self):
        def known_value(x):
            return self.value

        def field_value(x):
            return getattr(x, self.value.field)

        if isinstance(self.value, Ref):
            get_value = field_value
        else:
            get_value = known_value

        if self.kind == "Eq":
            return lambda x: getattr(x, self.field) == get_value(x)
        if self.kind == "In":
            return lambda x: getattr(x, self.field) in get_value(x)
        if self.kind == "Ins":
            return lambda x: [getattr(x, field) for field in self.field] in get_value(x)
        if self.kind == "Contains":
            return lambda x: get_value(x) in getattr(x, self.field)
        assert False, "unknown constraint kind"


def parse_constraint(polar, constraint):
    kind = constraint["kind"]
    assert kind in ["Eq", "In", "Contains"]
    field = constraint["field"]
    value = constraint["value"]

    if value is not None:
        value_kind = next(iter(value))
        value = value[value_kind]

        if value_kind == "Term":
            value = polar.host.to_python(value)
        elif value_kind == "Ref":
            value = Ref(result_id=value['result_id'], field=value['field'])
        else:
            assert False, "Unknown value kind"

    return Constraint(kind=kind, field=field, value=value)

def partition(coll, pred):
    yes, no = [], []
    for x in coll:
        (yes if pred(x) else no).append(x)
    return (yes, no)

def group_refs(cons):
    res = {}
    for con in cons:
        grp = res.get(con.value.result_id)
        if grp is None:
            grp = []
            res[con.value.result_id] = grp
        grp.append(con)
    return res

def ground_constraints(polar, results, filter_plan, constraints):
    refs, rest = partition(constraints, lambda c: isinstance(c.value, Ref) and c.value.result_id is not None)
    for rid, cons in group_refs(refs).items():
        fields = [[getattr(r, c.value.field) for c in cons] for r in results[rid]]
        rest.append(Constraint(value=fields, kind='Ins', field=[c.field for c in cons]))
    return rest


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
            constraints = ground_constraints(polar, set_results, filter_plan, constraints)
            fetcher = polar.host.fetchers[class_name]
            set_results[i] = fetcher(constraints)

        results.extend(set_results[result_id])

    # NOTE(steve): Not the best way to remove duplicates.
    return [i for n, i in enumerate(results) if i not in results[:n]]


def filter_data(polar, filter_plan, filter_plan_resolver=builtin_filter_plan_resolver):
    return filter_plan_resolver(polar, filter_plan)
