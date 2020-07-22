from polar.exceptions import UnrecognizedEOF
from oso import Oso, OsoException

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

import os

polar_file = os.path.dirname(os.path.realpath(__file__)) + "/test.polar"
oso.load_file(polar_file)
oso._load_queued_files()

assert oso.allow("a", "b", "c")

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

assert oso.query_predicate("specializers", D("hello"), B.C("hello")).success
assert oso.query_predicate("floatLists").success
assert oso.query_predicate("intDicts").success
assert oso.query_predicate("comparisons").success
assert oso.query_predicate("testForall").success
assert oso.query_predicate("testRest").success
assert oso.query_predicate("testMatches", A("hello")).success
assert oso.query_predicate("testMethodCalls", A("hello"), B.C("hello")).success
assert oso.query_predicate("testOr").success
assert oso.query_predicate("testHttpAndPathMapper").success

# Test that cut doesn't return anything.
assert oso.query_predicate("testCut").success is False

import math

# Test that a constant can be called.
oso.register_constant("Math", math)
oso.load_str("?= Math.factorial(5) == 120;")

