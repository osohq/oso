# External class definitions for use in `test_polar.py` tests


class Foo:
    def __init__(self, name=""):
        self.name = name

    def foo(self):
        return "Foo!"


class Bar(Foo):
    def foo(self):
        return "Bar!"


class Qux:
    pass


class MyClass:
    def __init__(self, x="", y=""):
        self.x = x
        self.y = y

    def __eq__(self, other):
        if isinstance(other, MyClass):
            return self.x == other.x and self.y == other.y
        return False


class YourClass:
    pass


class OurClass(MyClass, YourClass):
    pass
