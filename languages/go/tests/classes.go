package oso

import (
	"fmt"
	"reflect"

	oso "github.com/osohq/oso/languages/go/pkg"
)

type UnitClass struct{}

func (u UnitClass) String() string {
	return "UnitClass"
}

type IterableClass struct {
	Elems []int
}

func (ic IterableClass) Sum() int {
	res := 0
	for _, v := range ic.Elems {
		res += v
	}
	return res
}

func (ic IterableClass) Iter() chan interface{} {
	c := make(chan interface{})
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

func (vf ValueFactory) GetNil() *int {
	return nil
}

func (vf ValueFactory) GetString() string {
	return vf.StringAttr
}

func (vf ValueFactory) GetList() []int {
	return vf.ListAttr
}

func (vf ValueFactory) GetDict() map[string]int {
	return vf.DictAttr
}

func (vf ValueFactory) GetClass() error {
	return fmt.Errorf("unimplemented")
}

func (vf ValueFactory) GetInstance() error {
	// TODO: What does this return?
	return fmt.Errorf("unimplemented")
}

func (vf ValueFactory) GetType() reflect.Type {
	return reflect.TypeOf(vf.InnerClass)
}

type Constructor map[string]interface{}

func (u Constructor) String() string {
	return "Constructor"
}

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

func (c Constructor) NumKwargs() int {
	return len(map[string]interface{}(c))
}

type MethodVariants struct {
}

func (u MethodVariants) String() string {
	return "MethodVariants"
}

func (m MethodVariants) ClassMethodReturnsString() string {
	return "abc"
}

func (m MethodVariants) SumInputArgs(args ...int) int {
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

// TODO: I don't think these make sense. Maybe as interfaces?
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

func (a Animal) String() string {
	return fmt.Sprintf("Animal { %s, %s, %s }", a.Species, a.Genus, a.Family)
}

type ImplementsEq struct {
	Val int
}

func (u ImplementsEq) String() string {
	return fmt.Sprintf("ImplementsEq { %v }", u.Val)
}

func (left ImplementsEq) Equal(right oso.Comparer) bool {
	return left.Val == right.(ImplementsEq).Val
}
func (left ImplementsEq) Lt(right oso.Comparer) bool {
	panic("unsupported")
}

type Comparable struct {
	Val int
}

func (u Comparable) String() string {
	return fmt.Sprintf("Comparable { %v }", u.Val)
}

func (a Comparable) Equal(b oso.Comparer) bool {
	if other, ok := b.(Comparable); ok {
		return a.Val == other.Val
	}
	panic(fmt.Sprintf("cannot compare Comparable with %v", b))
}

func (a Comparable) Lt(b oso.Comparer) bool {
	if other, ok := b.(Comparable); ok {
		return a.Val < other.Val
	}
	panic(fmt.Sprintf("cannot compare Comparable with %v", b))
}
