package oso_test

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"reflect"
	"regexp"
	"strings"
	"testing"

	"github.com/osohq/go-oso/interfaces"
	"github.com/osohq/go-oso/internal/host"
	. "github.com/osohq/go-oso/types"

	yaml "github.com/goccy/go-yaml"
	"github.com/google/go-cmp/cmp"
	oso "github.com/osohq/go-oso"
)

type UnitClass struct{}

func NewUnitClass() UnitClass {
	return UnitClass{}
}

func (u UnitClass) String() string {
	return "UnitClass"
}

func (u UnitClass) New() UnitClass {
	return UnitClass{}
}

type IterableClass struct {
	Elems []int
}

func (ic IterableClass) New(elems []int) IterableClass {
	return IterableClass{Elems: elems}
}

func (ic IterableClass) Sum() int {
	res := 0
	for _, v := range ic.Elems {
		res += v
	}
	return res
}

func (ic IterableClass) Iter() <-chan interface{} {
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
	return NewValueFactory().StringAttr
}

func (vf ValueFactory) GetList() []int {
	return NewValueFactory().ListAttr
}

func (vf ValueFactory) GetDict() map[string]int {
	return NewValueFactory().DictAttr
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

func (c Constructor) NumKwargs() int {
	return len(map[string]interface{}(c))
}

type MethodVariants struct {
}

func (u MethodVariants) New() MethodVariants {
	return MethodVariants{}
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

func (MethodVariants) GetIter() interfaces.Iterator {
	return IterableClass{Elems: NewValueFactory().ListAttr}
}

func (MethodVariants) GetEmptyIter() interfaces.Iterator {
	return IterableClass{}
}

// TODO: I don't think these make sense. Maybe as interfaces?
type ParentClass struct{}

type ChildClass struct{}

type GrandchildClass struct{}

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

func (u ImplementsEq) New(val int) ImplementsEq {
	return ImplementsEq{Val: val}
}

func (u ImplementsEq) String() string {
	return fmt.Sprintf("ImplementsEq { %v }", u.Val)
}

func (left ImplementsEq) Equal(right interfaces.Comparer) bool {
	return left.Val == right.(ImplementsEq).Val
}
func (left ImplementsEq) Lt(right interfaces.Comparer) bool {
	panic("unsupported")
}

type Comparable struct {
	Val int
}

func (u Comparable) New(val int) Comparable {
	return Comparable{Val: val}
}

func (u Comparable) String() string {
	return fmt.Sprintf("Comparable { %v }", u.Val)
}

func (a Comparable) Equal(b interfaces.Comparer) bool {
	if other, ok := b.(Comparable); ok {
		return a.Val == other.Val
	}
	panic(fmt.Sprintf("cannot compare Comparable with %v", b))
}

func (a Comparable) Lt(b interfaces.Comparer) bool {
	if other, ok := b.(Comparable); ok {
		return a.Val < other.Val
	}
	panic(fmt.Sprintf("cannot compare Comparable with %v", b))
}

var CLASSES = map[string]reflect.Type{
	"UnitClass":       reflect.TypeOf(UnitClass{}),
	"ValueFactory":    reflect.TypeOf(ValueFactory{}),
	"IterableClass":   reflect.TypeOf(IterableClass{}),
	"Constructor":     reflect.TypeOf(Constructor{}),
	"MethodVariants":  reflect.TypeOf(MethodVariants{}),
	"ParentClass":     reflect.TypeOf(ParentClass{}),
	"ChildClass":      reflect.TypeOf(ChildClass{}),
	"GrandchildClass": reflect.TypeOf(GrandchildClass{}),
	"Animal":          reflect.TypeOf(Animal{}),
	"ImplementsEq":    reflect.TypeOf(ImplementsEq{}),
	"Comparable":      reflect.TypeOf(Comparable{}),
}

func setStructFields(instance reflect.Value, args []interface{}) error {
	for idx, arg := range args {
		f := instance.Field(idx)
		if !f.IsValid() {
			return fmt.Errorf("Cannot set field #%v", idx)
		}
		err := host.SetFieldTo(f, arg)
		if err != nil {
			return err
		}
	}
	return nil
}

func setMapFields(instance reflect.Value, kwargs map[string]interface{}) error {
	for k, v := range kwargs {
		f := instance.FieldByName(k)
		if !f.IsValid() {
			return fmt.Errorf("Cannot set field %v", k)
		}
		err := host.SetFieldTo(f, v)
		if err != nil {
			return err
		}
	}
	return nil
}

// InstantiateClass sets the fields of a new instance of `class` to those provided in `args` and `kwargs`
func instantiateClass(class reflect.Type, args []interface{}, kwargs map[string]interface{}) (*interface{}, error) {
	instancePtr := reflect.New(class)
	instance := instancePtr.Elem()

	switch class.Kind() {
	case reflect.Struct:
		err := setStructFields(instance, args)
		if err != nil {
			return nil, err
		}
		err = setMapFields(instance, kwargs)
		if err != nil {
			return nil, err
		}
	case reflect.Array, reflect.Slice:
		if len(kwargs) != 0 {
			return nil, fmt.Errorf("Cannot assign kwargs to a class of type: %s", class.Kind())
		}
		err := host.SetFieldTo(instance, args)
		if err != nil {
			return nil, err
		}
	case reflect.Map:
		if len(args) != 0 {
			return nil, fmt.Errorf("Cannot assign args to a class of type: %s", class.Kind())
		}
		err := host.SetFieldTo(instance, kwargs)
		if err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("Cannot instantiate a class of type: %s", class.Kind())
	}
	instanceInterface := instance.Interface()
	return &instanceInterface, nil
}

type TestCase struct {
	// Raw         string                   `yaml:omit`
	Name        string   `yaml:"name"`
	Description string   `yaml:"description"`
	Policies    []string `yaml:"policies"`
	Cases       []Case   `yaml:"cases"`
}

type Result struct {
	inner interface{}
}

func toResult(input map[string]interface{}) map[string]Result {
	result := make(map[string]Result)
	for k, v := range input {
		result[k] = NewResult(v)
	}
	return result
}

func NewResult(input interface{}) Result {
	switch inputVal := input.(type) {
	case map[string]interface{}:
		result := make(map[string]Result)
		for k, v := range inputVal {
			result[k] = NewResult(v)
		}
		return Result{result}
	case []interface{}:
		result := make([]Result, len(inputVal))
		for idx, v := range inputVal {
			result[idx] = NewResult(v)
		}
		return Result{result}
	case uint64:
		// standardise uints to ints
		return Result{inner: inputVal}
	default:
		return Result{input}
	}
}

func (left Result) Equal(right interface{}) bool {
	switch val := left.inner.(type) {
	case map[string]Result:
		if repr, ok := val["repr"]; ok {
			return repr.inner.(string) == fmt.Sprintf("%v", right)
		}
	}
	return cmp.Equal(left.inner, right)
}

func toInput(v interface{}, t *testing.T) interface{} {
	if vMap, ok := v.(map[string]interface{}); ok {
		if ty, ok := vMap["type"]; ok {
			class := CLASSES[ty.(string)]
			if class == nil {
				t.Fatalf("class %s not implemented for tests", ty.(string))
			}
			var args []interface{}
			var kwargs map[string]interface{}
			if vMap["args"] != nil {
				args = vMap["args"].([]interface{})
			}
			if vMap["kwargs"] != nil {
				kwargs = vMap["kwargs"].(map[string]interface{})
			}
			instance, err := instantiateClass(class, args, kwargs)
			if err != nil {
				t.Fatal(err)
			}
			return instance
		}
		if v, ok := vMap["var"]; ok {
			return ValueVariable(v.(string))
		}
	}
	return v
}

type Case struct {
	Description *string                   `yaml:"description"`
	Query       string                    `yaml:"query"`
	Load        *string                   `yaml:"load"`
	Inputs      *[]interface{}            `yaml:"input"`
	Result      *[]map[string]interface{} `yaml:"result"`
	Err         *string                   `yaml:"err"`
}

func contains(list []string, elem string) bool {
	for _, a := range list {
		if a == elem {
			return true
		}
	}
	return false
}

func String(s string) *string {
	return &s
}

func (tc TestCase) setupTest(o oso.Oso, t *testing.T) error {
	var CONSTRUCTORS = map[string]interface{}{
		"UnitClass":    NewUnitClass,
		"ValueFactory": NewValueFactory,
	}
	for k, v := range CLASSES {
		c := CONSTRUCTORS[k]
		err := o.RegisterClassWithName(v, c, k)
		if err != nil {
			t.Fatal(err)
		}
	}

	var policies_folder string
	if _, err := os.Stat("policies"); !os.IsNotExist(err) {
		policies_folder = "policies/"
	} else {
		policies_folder = "../../../test/policies/"
	}

	err := filepath.Walk(policies_folder, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}

		if contains(tc.Policies, strings.TrimSuffix(filepath.Base(path), filepath.Ext(path))) {
			err = o.LoadFile(path)
			if err != nil {
				return err
			}
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	return nil
}

func (tc TestCase) RunTest(t *testing.T) {
	for _, c := range tc.Cases {
		testName := tc.Name + "\n" + tc.Description + "\n"
		if c.Description != nil {
			testName += *c.Description
		} else {
			testName += c.Query
		}
		t.Run(testName, func(t *testing.T) {
			name := t.Name()
			_ = name
			var o oso.Oso
			var err error
			if o, err = oso.NewOso(); err != nil {
				t.Fatalf("Failed to setup Oso: %s", err.Error())
			}
			err = tc.setupTest(o, t)
			if err != nil {
				t.Fatal(err)
			}
			var testQuery *oso.Query
			var queryErr error
			if c.Inputs == nil {
				testQuery, queryErr = o.NewQueryFromStr(c.Query)
			} else {
				Inputs := make([]interface{}, len(*c.Inputs))
				for idx, v := range *c.Inputs {
					input := toInput(v, t)
					Inputs[idx] = input
				}
				testQuery, queryErr = o.NewQueryFromRule(c.Query, Inputs...)
			}

			expectedResults := make([]map[string]Result, 0)
			if c.Result == nil {
				expectedResults = append(expectedResults, toResult(make(map[string]interface{})))
			} else {
				for _, v := range *c.Result {
					expectedResults = append(expectedResults, toResult(v))
				}
			}

			if c.Load != nil {
				queryErr = o.LoadString(*c.Load)
			}

			var results []map[string]interface{}
			if queryErr == nil {
				results, queryErr = testQuery.GetAllResults()
			}

			if c.Err != nil {
				if queryErr != nil {
					re, err := regexp.Compile(*c.Err)
					if err != nil {
						t.Error(err)
					}
					if !re.Match([]byte(queryErr.Error())) {
						t.Error(fmt.Errorf("expected query to fail with:\n\t\"%s\"\nGot:\n\t\"%s\"", *c.Err, queryErr))
					}
				} else {
					t.Error(fmt.Errorf("expected query to fail with:\n\t\"%s\". Got:\n\tSuccess", *c.Err))
				}
			} else {
				if queryErr != nil {
					t.Error(queryErr)
				} else {
					if len(results) != len(expectedResults) {
						t.Error(fmt.Errorf("incorrect number of results\nGot: %v\nExpected: %v\n:\n%s", len(results), len(expectedResults), cmp.Diff(expectedResults, results)))
						return
					}
					for idx, expectedResult := range expectedResults {
						result := results[idx]
						for k, v := range expectedResult {
							if v2, ok := result[k]; ok {
								if !cmp.Equal(v2, v2) {
									t.Error(fmt.Errorf("incorrect query result:\n%s", cmp.Diff(v2, v)))
								}
							} else {
								t.Error(fmt.Errorf("missing query result for: %v\n%s", k, cmp.Diff(v, nil)))
							}
						}
					}
				}

			}
		})
	}
}

func testFromFile(t *testing.T, path string) {
	yamlInput, err := ioutil.ReadFile(path)
	if err != nil {
		t.Error(err)
	}
	var testCase TestCase
	err = yaml.Unmarshal(yamlInput, &testCase)
	if err != nil {
		t.Fatal(err)
	}
	testCase.RunTest(t)
}

func TestAll(t *testing.T) {
	var spec_folder string
	if _, err := os.Stat("spec"); !os.IsNotExist(err) {
		spec_folder = "spec/"
	} else {
		spec_folder = "../../../test/spec/"
	}
	err := filepath.Walk(spec_folder, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		testFromFile(t, path)
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
}
