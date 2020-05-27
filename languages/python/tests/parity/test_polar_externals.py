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


class Person:
    def __init__(self, name=""):
        self.name = name


class Employee(Person):
    def __init__(self, manager=None):
        self.manager = manager


class Manager(Employee):
    def __init__(self, id=0, name="", manager=None):
        self.name = name
        self.id = id
        self.manager = manager
