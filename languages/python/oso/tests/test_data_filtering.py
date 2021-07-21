import pytest

from typing import Any, ClassVar
from dataclasses import dataclass
from oso import Oso, OsoError
from polar import Relationship

from polar.expression import Expression, Pattern
from polar.partial import Variable

from polar.data_filtering import (
    filter_data,
    ground_constraints,
    Constraints,
    Constraint,
    Attrib,
    Result,
    FilterPlan,
    process_constraints,
)


@pytest.fixture
def oso():
    oso = Oso()
    return oso


def test_data_filtering(oso):
    # Register some types and callbacks
    @dataclass
    class Bar:
        id: str
        is_cool: bool
        is_still_cool: bool

    @dataclass
    class Foo:
        id: str
        bar_id: str
        is_fooey: bool

    hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
    goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
    something_foo = Foo(id="something", bar_id="hello", is_fooey=False)
    another_foo = Foo(id="another", bar_id="hello", is_fooey=True)
    third_foo = Foo(id="third", bar_id="hello", is_fooey=True)
    forth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True)

    bars = [hello_bar, goodbye_bar]
    foos = [something_foo, another_foo, third_foo, forth_foo]

    def matches_fields(fields, obj):
        for k, v in fields.items():
            if getattr(obj, k) != v:
                return False
            return True

    def field_matcher(fields):
        def matcher(obj):
            return matches_fields(fields, obj)

        return matcher

    def get_bars(constraints):
        results = []
        assert constraints.cls == Bar
        for bar in bars:
            matches = True
            for constraint in constraints.constraints:
                val = getattr(bar, constraint.field)
                if constraint.kind == "Eq":
                    if val != constraint.value:
                        matches = False
                        break
                if constraint.kind == "In":
                    if val not in constraint.value:
                        matches = False
                        break
            if matches:
                results.append(bar)
        return results

    def get_foos(constraints):
        results = []
        assert constraints.cls == Foo
        for foo in foos:
            matches = True
            for constraint in constraints.constraints:
                val = getattr(foo, constraint.field)
                if constraint.kind == "Eq":
                    if val != constraint.value:
                        matches = False
                        break
                if constraint.kind == "In":
                    if val not in constraint.value:
                        matches = False
                        break
            if matches:
                results.append(foo)
        return results

    oso.register_class(Bar, types={"id": str, "is_cool": bool}, fetcher=get_bars)
    oso.register_class(
        Foo,
        types={
            "id": str,
            "bar_id": str,
            "bar": Relationship(
                kind="parent", other_type=Bar, my_field="bar_id", other_field="id"
            ),
        },
        fetcher=get_foos,
    )

    # Write a policy
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", another_foo)

    # So, for my first query, I would get something like this.
    plan = FilterPlan(
        {1: Constraints(Foo, [Constraint("Eq", "is_fooey", True)])}, [1], 1
    )
    results = filter_data(oso, plan)
    assert len(results) == 3

    # Test process constraints
    # This is what comes back from the partial
    query_results = [
        {
            "bindings": {
                "resource": Expression(
                    "And",
                    [
                        Expression("Isa", [Variable("_this"), Pattern(Foo, {})]),
                        Expression(
                            "Unify",
                            [True, Expression("Dot", [Variable("_this"), "is_fooey"])],
                        ),
                    ],
                )
            },
            "trace": None,
        }
    ]

    processed = process_constraints(oso, Foo, "resource", query_results)
    assert processed == plan

    # Once I add the actual hard part too.
    results = list(oso.get_allowed_resources("steve", "get", Foo))
    assert len(results) == 3

    oso.clear_rules()
    #
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.bar = bar and
        bar.is_cool = true and
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", another_foo)

    # The second one would look like this
    plan2 = FilterPlan(
        {
            1: Constraints(
                Foo,
                [
                    Constraint("In", "bar_id", Attrib("id", Result(2))),
                    Constraint("Eq", "is_fooey", True),
                ],
            ),
            2: Constraints(Bar, [Constraint("Eq", "is_cool", True)]),
        },
        [2, 1],
        1,
    )
    results = filter_data(oso, plan2)
    assert len(results) == 2

    query_results = [
        {
            "bindings": {
                "resource": Expression(
                    "And",
                    [
                        Expression("Isa", [Variable("_this"), Pattern(Foo, {})]),
                        Expression(
                            "Unify",
                            [
                                True,
                                Expression(
                                    "Dot",
                                    [
                                        Expression("Dot", [Variable("_this"), "bar"]),
                                        "is_cool",
                                    ],
                                ),
                            ],
                        ),
                        Expression(
                            "Unify",
                            [True, Expression("Dot", [Variable("_this"), "is_fooey"])],
                        ),
                    ],
                )
            },
            "trace": None,
        }
    ]

    # processed = process_constraints(oso, Foo, "resource", query_results)
    # assert processed == plan2

    results = list(oso.get_allowed_resources("steve", "get", Foo))
    assert len(results) == 2
