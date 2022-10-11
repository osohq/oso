from typing import List


def test_unhandled_partial_gh1467():
    """Test that previously failed due to incorrect partial unification.

    Submitted by BAH.

    Fixed in https://github.com/osohq/oso/pull/1467."""
    POLICY = """\
actor User {}

allow(actor, action, resource) if has_permission(actor, action, resource);

resource A {
    permissions = ["Read"];
    roles = ["User"];

    "Read" if "User";
}

has_role(user: User, "User", a: A) if
    a_role in a.groups and
    "Read" = a_role.p and
    a_role.group_id in user.group_ids;

resource Aprime {
    relations = { a: A };
    permissions = ["Read"];

    "Read" if "User" on "a";
}

has_relation(subject: A, "a", object: Aprime) if
    subject = object;
    """

    from dataclasses import dataclass

    from oso import Oso, Variable
    from polar import Expression, Pattern

    @dataclass
    class Group:
        permission: str
        group_id: int

    @dataclass
    class A:
        groups: List[Group]

    @dataclass
    class Aprime(A):
        pass

    @dataclass
    class User:
        group_ids: List[int]

    oso = Oso()
    oso.register_class(Aprime)
    oso.register_class(A)
    oso.register_class(User)

    oso.load_str(POLICY)

    constraint = Expression(
        "And",
        [Expression("Isa", [Variable("resource"), Pattern("Aprime", {})])],
    )

    # Unhandled partial was raised here before fix.
    results = list(
        oso.query_rule(
            "allow",
            User(group_ids=[0]),
            "Read",
            Variable("resource"),
            accept_expression=True,
            bindings={"resource": constraint},
        )
    )
    print(results)
