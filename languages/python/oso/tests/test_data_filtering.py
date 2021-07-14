import pytest

from typing import Any, ClassVar
from dataclasses import dataclass
from oso import Oso, OsoError
from polar import Relationship


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

    @dataclass
    class Foo:
        id: str
        bar_id: str

    hello_bar = Bar(id="hello", is_cool=True)
    goodbye_bar = Bar(id="goodbye", is_cool=False)
    something_foo = Foo(id="something", bar_id="hello")

    bars = [hello_bar, goodbye_bar]
    foos = [something_foo]

    # I think these probably take a list of dicts of fields to match
    # Should be able to like get them all at once if you want to but
    # also they are separate so not sure the api exactly yet.
    # These are just like brute force search of an array.
    def get_bars(queries):
        results = []
        for fields in queries:
            result = []
            for bar in bars:
                valid = True
                for k, v in fields.items():
                    if getattr(bar, k) != v:
                        valid = False
                        break
                if valid:
                    result.append(bar)
            results.append(result)
        return results

    def get_foos(queries):
        results = []
        for fields in queries:
            result = []
            for foo in foos:
                valid = True
                for k, v in fields.items():
                    if getattr(foo, k) != v:
                        valid = False
                        break
                if valid:
                    result.append(foo)
            results.append(result)
        return results

    oso.register_class(Bar, types={"id": str, "is_cool": bool}, fetcher=get_bars)
    oso.register_class(
        Foo,
        types={
            "id": str,
            "bar_id": str,
            "bar": Relationship(
                kind="many-to-one", other_type=Bar, my_field="bar_id", other_field="id"
            ),
        },
        fetcher=get_foos,
    )

    # Write a policy
    policy = """
    allow("steve", "get", resource: Foo) if
        bar = resource.bar and
        bar.is_cool = true;
    """
    oso.load_str(policy)
    # Call some new data filtering method

    # Try a normal is_allowed()
    assert oso.is_allowed("steve", "get", something_foo)

    results = list(oso.query('allow("steve", "get", foo)', accept_expression=True))
    print(results)

    # oso.get_allowed_resources(user, action, cls)
