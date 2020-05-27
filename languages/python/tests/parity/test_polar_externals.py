# External class definitions for use in `test_polar.py` tests


class Foo:
    def __init__(self, name=""):
        self.name = name

    def foo(self):
        yield "Foo!"


class Bar(Foo):
    def foo(self):
        yield "Bar!"


class Qux:
    pass


class MyClass:
    def __init__(self, x="", y=""):
        self.x = x
        self.y = y


class YourClass:
    pass


class OurClass(MyClass, YourClass):
    pass
