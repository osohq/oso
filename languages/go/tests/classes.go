package oso

import (
	"fmt"
	"reflect"
)

type UnitClass struct{}

func (u UnitClass) String() string {
	return "UnitClass"
}

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

type Constructor map[string]interface{}

// func NewConstructor(args ...interface{}) Constructor {
// 	Args := make([]interface{}, len(args))
// 	for idx, v := range args {
// 		Args[idx] = v
// 	}
// 	return Constructor{
// 		Args:   Args,
// 		Kwargs: make(map[string]interface{}),
// 	}
// }

// func (c Constructor) numArgs() int {
// 	return len(c.Args)
// }

func (c Constructor) numKwrgs() int {
	return len(map[string]interface{}(c))
}

type MethodVariants struct {
}

func (m *MethodVariants) class_method_return_string() string {
	return "abc"
}

func (m *MethodVariants) sum_input_args(args ...int) int {
	sum := 0
	for _, arg := range args {
		sum += arg
	}
	return sum
}

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

type ParentClass struct{}

// class ParentClass:
//     def inherit_parent(self):
//         return "parent"

//     def override_parent(self):
//         return "parent"

type ChildClass struct{}

// class ChildClass(ParentClass):
//     def inherit_child(self):
//         return "child"

//     def override_parent(self):
//         return "child"

type GrandchildClass struct{}

// class GrandchildClass(ChildClass):
//     def inherit_grandchild(self):
//         return "grandchild"

//     def override_parent(self):
//         return "grandchild"

type Animal struct {
	Species string
	Genus   string
	Family  string
}

type ImplementsEq struct {
	val int
}

func (left ImplementsEq) Equal(right ImplementsEq) bool {
	return left.val == right.val
}

type Comparable struct {
	Val int
}

func (a Comparable) Compare(b Comparable) int {
	if a.Val > b.Val {
		return 1
	}
	if a.Val < b.Val {
		return -1
	}
	return 0
}
