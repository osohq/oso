import pytest
from dataclasses import dataclass
from oso import Relation
from helpers import *

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
    numbers: list

@dataclass
class Log:
    id: str
    foo_id: str
    data: str

hello_bar = Bar(id="hello", is_cool=True, is_still_cool=True)
goodbye_bar = Bar(id="goodbye", is_cool=False, is_still_cool=True)
hershey_bar = Bar(id="hershey", is_cool=False, is_still_cool=False)

something_foo = Foo(id="something", bar_id="hello", is_fooey=False, numbers=[])
another_foo = Foo(id="another", bar_id="hello", is_fooey=True, numbers=[1])
third_foo = Foo(id="third", bar_id="hello", is_fooey=True, numbers=[2])
fourth_foo = Foo(id="fourth", bar_id="goodbye", is_fooey=True, numbers=[2, 1])

fourth_log_a = Log(id="a", foo_id="fourth", data="hello")
third_log_b = Log(id="b", foo_id="third", data="world")
another_log_c = Log(id="c", foo_id="another", data="steve")

# Shared test setup.
@pytest.fixture
def t(oso):

    bars = [hello_bar, goodbye_bar, hershey_bar]
    foos = [something_foo, another_foo, third_foo, fourth_foo]
    foo_logs = [fourth_log_a, third_log_b, another_log_c]

    def get_bars(constraints):
        return filter_array(bars, constraints)

    def get_foos(constraints):
        return filter_array(foos, constraints)

    def get_foo_logs(constraints):
        return filter_array(foo_logs, constraints)

    # Combining is combining but filtering out duplicates.
    def combine_query(q1, q2):
        results = q1 + q2
        return [i for n, i in enumerate(results) if i not in results[:n]]

    oso.set_data_filtering_query_defaults(
        exec_query=lambda results: results, combine_query=combine_query
    )

    oso.register_class(
        Bar,
        fields={
            "id": str,
            "is_cool": bool,
            "is_still_cool": bool,
            "foos": Relation(
                kind="many", other_type="Foo", my_field="id", other_field="bar_id"
            ),
        },
        build_query=get_bars,
    )
    oso.register_class(
        Foo,
        fields={
            "id": str,
            "bar_id": str,
            "is_fooey": bool,
            "numbers": list,
            "bar": Relation(
                kind="one", other_type="Bar", my_field="bar_id", other_field="id"
            ),
            "logs": Relation(
                kind="many",
                other_type="Log",
                my_field="id",
                other_field="foo_id",
            ),
        },
        build_query=get_foos,
    )
    oso.register_class(
        Log,
        fields={
            "id": str,
            "foo_id": str,
            "data": str,
            "foo": Relation(
                kind="one", other_type="Foo", my_field="foo_id", other_field="id"
            ),
        },
        build_query=get_foo_logs,
    )
    # Sorta hacky, just return anything you want to use in a test.
    return {
        "Foo": Foo,
        "Bar": Bar,
        "Log": Log,
        "another_foo": another_foo,
        "third_foo": third_foo,
        "something_foo": something_foo,
        "fourth_foo": fourth_foo,
        "fourth_log_a": fourth_log_a,
        "third_log_b": third_log_b,
        "another_log_c": another_log_c,
        "bars": bars,
        "foos": foos,
        "logs": foo_logs,
    }


