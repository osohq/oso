from oso import Oso

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
