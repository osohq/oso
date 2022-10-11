from functools import reduce

from oso import Oso


class DfTestOso(Oso):
    def check_authz(self, actor, action, resource, expected):
        assert unord_eq(self.authorized_resources(actor, action, resource), expected)
        for re in expected:
            assert self.is_allowed(actor, action, re)


def unord_eq(a, b):
    b = list(b)
    for x in a:
        try:
            b.remove(x)
        except ValueError:
            return False
    return not b


def filter_array(array):
    def go(constraints):
        check = reduce(
            lambda f, g: lambda x: f(x) and g(x),
            [c.check for c in constraints],
            lambda _: True,
        )
        return [x for x in array if check(x)]

    return go
