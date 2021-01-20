from oso import Variable, Predicate

class UnitClass:
    """Simple class"""

    def __repr__(self):
        return "UnitClass"


class IterableClass(list):
    """Class that can be iterated over"""

    def sum(self):
        return sum(x for x in self)

class ValueFactory:
    """Returns basic value types from host"""
    string_attr = "abc"
    list_attr = [1, 2, 3]
    dict_attr = {"a": 1, "b": 2}

    class InnerClass:
        pass

    def get_nil(self):
        return None

    def get_string(self):
        return self.string_attr
    
    def get_list(self):
        return self.list_attr

    def get_dict(self):
        return self.dict_attr

    def get_class(self):
        return self.InnerClass
    
    def get_instance(self):
        return self.InnerClass()

    def get_type(self):
        return type(self.InnerClass)

class Constructor:
    def __repr__(self):
        return "Constructor"

    def __init__(self, *args, **kwargs):
        """For testing constructor"""
        self.args = args
        self.kwargs = kwargs

    def num_args(self):
        return len(self.args)

    def num_kwargs(self):
        return len(self.kwargs)


class MethodVariants:
    """Class with various method variants"""

    @classmethod
    def class_method_return_string(cls):
        return "abc"

    @classmethod
    def sum_input_args(cls, *args):
        return sum(args)

    def is_key_in_kwargs(self, key, **kwargs):
        return key in kwargs

    def set_x_or_y(self, x=1, y=2):
        return [x, y]

    def get_iter(self):
        return iter(ValueFactory.list_attr)

    def get_empty_iter(self):
        return iter([])

    def get_generator(self):
        yield from iter(ValueFactory.list_attr)
    
    def get_empty_generator(self):
        yield from iter([])
    
class ParentClass:
    def inherit_parent(self):
        return "parent"

    def override_parent(self):
        return "parent"

class ChildClass(ParentClass):
    def inherit_child(self):
        return "child"

    def override_parent(self):
        return "child"

class GrandchildClass(ChildClass):
    def inherit_grandchild(self):
        return "grandchild"

    def override_parent(self):
        return "grandchild"


class Animal:
    """Class to check dictionary specializers"""
    def __init__(self, species=None, genus=None, family=None):
        self.genus = genus
        self.species = species
        self.family = family

class ImplementsEq:
    def __init__(self, val):
        self.val = val

    def __eq__(self, other):
        return isinstance(other, ImplementsEq) and self.val == other.val

class Comparable:
    def __init__(self, val):
        self.val = val

    def __gt__(self, other):
        return self.val > other.val

    def __lt__(self, other):
        return self.val < other.val

    def __eq__(self, other):
        return self.val == other.val

    def __le__(self, other):
        return self < other or self == other
    
    def __ge__(self, other):
        return self > other or self == other
