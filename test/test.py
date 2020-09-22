import math
import os

from polar.exceptions import UnrecognizedEOF
from oso import Oso, OsoError, Variable

oso = Oso()

# Application class with default kwargs constructor, registered with the
# decorator.
class A:
    def __init__(self, x):
        self.x = x

    def foo(self):
        return -1


oso.register_class(A)


# Test inheritance; doesn't need to be registered.
class D(A):
    pass


# Namespaced application class (to be aliased) with custom
# constructor.
class B:
    class C:
        def __init__(self, y):
            self.y = y

        def foo(self):
            return -1


def custom_c_constructor(y):
    return B.C(y)


oso.register_class(B.C, name="C", from_polar=custom_c_constructor)


class E:
    @staticmethod
    def sum(*args):
        return sum(*args)


oso.register_class(E)

polar_file = os.path.dirname(os.path.realpath(__file__)) + "/test.polar"
oso.load_file(polar_file)

assert oso.is_allowed("a", "b", "c")

# Test that a built in string method can be called.
oso.load_str("""?= x = "hello world!" and x.endswith("world!");""")

# Test that a custom error type is thrown.
exception_thrown = False
try:
    oso.load_str("missingSemicolon()")
except UnrecognizedEOF as e:
    exception_thrown = True
    assert (
        str(e)
        == "hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 19"
    )
assert exception_thrown

assert list(oso.query_rule("specializers", D("hello"), B.C("hello")))
assert list(oso.query_rule("floatLists"))
assert list(oso.query_rule("intDicts"))
assert list(oso.query_rule("comparisons"))
assert list(oso.query_rule("testForall"))
assert list(oso.query_rule("testRest"))
assert list(oso.query_rule("testMatches", A("hello")))
assert list(oso.query_rule("testMethodCalls", A("hello"), B.C("hello")))
assert list(oso.query_rule("testOr"))
assert list(oso.query_rule("testHttpAndPathMapper"))
assert list(oso.query_rule("testUnifyClass", A))

# Test that cut doesn't return anything.
assert not list(oso.query_rule("testCut"))

# Test that a constant can be called.
oso.register_constant("Math", math)
oso.load_str("?= Math.factorial(5) == 120;")

# Test built-in type specializers.
assert list(oso.query('builtinSpecializers(true, "Boolean")'))
assert not list(oso.query('builtinSpecializers(false, "Boolean")'))
assert list(oso.query('builtinSpecializers(2, "Integer")'))
assert list(oso.query('builtinSpecializers(1, "Integer")'))
assert not list(oso.query('builtinSpecializers(0, "Integer")'))
assert not list(oso.query('builtinSpecializers(-1, "Integer")'))
assert list(oso.query('builtinSpecializers(1.0, "Float")'))
assert not list(oso.query('builtinSpecializers(0.0, "Float")'))
assert not list(oso.query('builtinSpecializers(-1.0, "Float")'))
assert list(oso.query('builtinSpecializers(["foo", "bar", "baz"], "List")'))
assert not list(oso.query('builtinSpecializers(["bar", "foo", "baz"], "List")'))
assert list(oso.query('builtinSpecializers({foo: "foo"}, "Dictionary")'))
assert not list(oso.query('builtinSpecializers({foo: "bar"}, "Dictionary")'))
assert list(oso.query('builtinSpecializers("foo", "String")'))
assert not list(oso.query('builtinSpecializers("bar", "String")'))

# Test deref works
oso.load_str("?= x = 1 and E.sum([x, 2, x]) = 4 and [3, 2, x].index(1) = 2;")

# Test unspecialized rule ordering
result = oso.query_rule("testUnspecializedRuleOrder", "foo", Variable("y"))
assert next(result)["bindings"]["y"] == 1
assert next(result)["bindings"]["y"] == 2
result = oso.query_rule("testUnspecializedRuleOrder", "foo", Variable("x"))
assert next(result)["bindings"]["x"] == 1
assert next(result)["bindings"]["x"] == 2
result = oso.query_rule("testUnspecializedRuleOrder", "foo", "bar", Variable("z"))
assert next(result)["bindings"]["z"] == 1
assert next(result)["bindings"]["z"] == 2
