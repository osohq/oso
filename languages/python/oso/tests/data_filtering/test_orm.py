from helpers import *
from orm_examples import *

def test_sqlalchemy_relationship(oso, sqlalchemy_t):
    policy = """
    allow("steve", "get", resource: Foo) if
        resource.bar = bar and
        bar.is_cool = true and
        resource.is_fooey = true;
    """
    oso.load_str(policy)
    assert oso.is_allowed("steve", "get", sqlalchemy_t["another_foo"])

    results = oso.authorized_resources("steve", "get", sqlalchemy_t["Foo"])
    assert len(results) == 2


def test_sqlalchemy_neq(oso, sqlalchemy_t):
    policy = """
    allow("steve", "get", foo: Foo) if foo.bar.id != "hello";
    allow("steve", "put", foo: Foo) if foo.bar.id != "goodbye";
    """
    oso.load_str(policy)
    t = sqlalchemy_t
    check_authz(oso, "steve", "get", t["Foo"], [t["fourth_foo"]])
    check_authz(
        oso,
        "steve",
        "put",
        t["Foo"],
        [t["another_foo"], t["third_foo"], t["something_foo"]],
    )


