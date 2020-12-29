package oso

import (
	"fmt"
	"reflect"
)

type UnitClass struct{}

type IterableClass struct {
	Elems []int
}

func (ic IterableClass) sum() int {
	res := 0
	for _, v := range ic.Elems {
		res += v
	}
	return res
}

func (ic IterableClass) Iter() chan int {
	c := make(chan int)
	go func() {
		for _, v := range ic.Elems {
			c <- v
		}
		close(c)
	}()
	return c
}

type ValueFactory struct {
	StringAttr string
	ListAttr   []int
	DictAttr   map[string]int
	InnerClass struct{}
}

func NewValueFactory() ValueFactory {
	return ValueFactory{
		StringAttr: "abc",
		ListAttr:   []int{1, 2, 3},
		DictAttr:   map[string]int{"a": 1, "b": 2},
	}
}

func (vf ValueFactory) get_nil() *int {
	return nil
}

func (vf ValueFactory) get_string() string {
	return vf.StringAttr
}

func (vf ValueFactory) get_list() []int {
	return vf.ListAttr
}

func (vf ValueFactory) get_dict() map[string]int {
	return vf.DictAttr
}

func (vf ValueFactory) get_class() error {
	return fmt.Errorf("unimplemented")
}

func (vf ValueFactory) get_instance() error {
	// TODO: What does this return?
	return fmt.Errorf("unimplemented")
}

func (vf ValueFactory) get_type() reflect.Type {
	return reflect.TypeOf(vf.InnerClass)
}

// class Constructor:
//     def __repr__(self):
//         return "Constructor"

//     def __init__(self, *args, **kwargs):
//         """For testing constructor"""
//         self.args = args
//         self.kwargs = kwargs

//     def num_args(self):
//         return len(self.args)

//     def num_kwargs(self):
//         return len(self.kwargs)

// class MethodVariants:
//     """Class with various method variants"""

//     @classmethod
//     def class_method_return_string(cls):
//         return "abc"

//     @classmethod
//     def sum_input_args(cls, *args):
//         return sum(args)

//     def is_key_in_kwargs(self, key, **kwargs):
//         return key in kwargs

//     def set_x_or_y(self, x=1, y=2):
//         return [x, y]

//     def get_iter(self):
//         return iter(ValueFactory.list_attr)

//     def get_empty_iter(self):
//         return iter([])

//     def get_generator(self):
//         yield from iter(ValueFactory.list_attr)

//     def get_empty_generator(self):
//         yield from iter([])

// class ParentClass:
//     def inherit_parent(self):
//         return "parent"

//     def override_parent(self):
//         return "parent"

// class ChildClass(ParentClass):
//     def inherit_child(self):
//         return "child"

//     def override_parent(self):
//         return "child"

// class GrandchildClass(ChildClass):
//     def inherit_grandchild(self):
//         return "grandchild"

//     def override_parent(self):
//         return "grandchild"

// class Animal:
//     """Class to check dictionary specializers"""
//     def __init__(self, species=None, genus=None, family=None):
//         self.genus = genus
//         self.species = species
//         self.family = family

// class ImplementsEq:
//     def __init__(self, val):
//         self.val = val

//     def __eq__(self, other):
//         return isinstance(other, ImplementsEq) and self.val == other.val

// class Comparable:
//     def __init__(self, val):
//         self.val = val

//     def __gt__(self, other):
//         return self.val > other.val

//     def __lt__(self, other):
//         return self.val < other.val

//     def __eq__(self, other):
//         return self.val == other.val

//     def __le__(self, other):
//         return self < other or self == other

//     def __ge__(self, other):
//         return self > other or self == other
