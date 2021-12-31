package oso_test

import (
	"reflect"
	"strings"
	"testing"

	oso "github.com/osohq/go-oso"
	"github.com/osohq/go-oso/internal/ffi"
	"github.com/osohq/go-oso/internal/host"
	. "github.com/osohq/go-oso/types"
)

// TEST oso.go

func TestNewOso(t *testing.T) {
	if o, err := oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	} else if reflect.TypeOf(o) != reflect.TypeOf(oso.Oso{}) {
		t.Fatalf("Expected type oso.Oso, got: %v", reflect.TypeOf(o))
	}
}

func TestLoadFiles(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if err = o.LoadFiles([]string{"test.polar", "test.polar"}); err == nil {
		t.Error("Failed to error on loading duplicate file")
	}

	if err = o.LoadFiles([]string{"test.txt"}); err == nil {
		t.Error("Failed to error on loading non-polar file (.txt)")
	}

	if err = o.LoadFiles([]string{"fake.polar"}); err == nil {
		t.Error("Failed to error on loading non-existent file")
	}
}

// test_load_multiple_files_same_name_different_path
func TestLoadMultipleFilesSameNameDifferentPath(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if err = o.LoadFiles([]string{"other/test.polar", "test.polar"}); err != nil {
		t.Error(err.Error())
	}

	expected := []map[string]interface{}{{"x": int64(1)}, {"x": int64(2)}, {"x": int64(3)}}

	for _, query := range []string{"f(x)", "g(x)"} {
		if testQuery, err := o.NewQueryFromStr(query); err != nil {
			t.Error(err.Error())
		} else {
			if results, err := testQuery.GetAllResults(); err != nil {
				t.Error(err.Error())
			} else {
				if len(results) != 3 {
					t.Errorf("Expected 3 results; received: %v", len(results))
				} else {
					for i, e := range expected {
						if !reflect.DeepEqual(results[i], e) {
							t.Errorf("Expected: %v, got: %v", e, results[i])
						}
					}
				}
			}
		}
	}
}

func TestLoadString(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if err = o.LoadString("f(1);"); err != nil {
		t.Error(err.Error())
	}

}

func TestClearRules(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.LoadString("f(1);")
	o.ClearRules()
	results, errors := o.QueryStr("f(x)")

	if err = <-errors; err == nil {
		t.Error("Expected query for undefined rule to throw error")
	}
	if !strings.Contains(err.Error(), "Query for undefined rule `f`") {
		t.Errorf("Received error does not match expected type: %v", err)
	}

	if r := <-results; r != nil {
		t.Error("Got result; expected none")
	}
}

func TestQueryStr(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.LoadString("f(1);")
	results, errors := o.QueryStr("f(x)")

	if err = <-errors; err != nil {
		t.Error(err.Error())
	} else {
		var got []map[string]interface{}
		expected := map[string]interface{}{"x": int64(1)}
		for elem := range results {
			got = append(got, elem)
		}
		if len(got) > 1 {
			t.Errorf("Received too many results: %v", got)
		} else if !reflect.DeepEqual(got[0], expected) {
			t.Errorf("Expected: %v, got: %v", expected, got[0])
		}
	}

	o.ClearRules()

	o.LoadString("g(x) if x.Fake();")
	results, errors = o.QueryStr("g(1)")

	if err = <-errors; err == nil {
		t.Error("Expected Polar runtime error, got none")
	}
}

func TestQueryRule(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.LoadString("f(1, 2);")
	results, errors := o.QueryRule("f", Variable("x"), Variable("y"))

	if err = <-errors; err != nil {
		t.Error(err.Error())
	} else {
		var got []map[string]interface{}
		expected := map[string]interface{}{"x": int64(1), "y": int64(2)}
		for elem := range results {
			got = append(got, elem)
		}
		if len(got) != 1 {
			t.Errorf("Received incorrect number of results: %v", got)
		} else if !reflect.DeepEqual(got[0], expected) {
			t.Errorf("Expected: %v, got: %v", expected, got[0])
		}
	}

	o.ClearRules()

	o.LoadString("g(x) if x.Fake();")
	results, errors = o.QueryRule("g", 1)

	if err = <-errors; err == nil {
		t.Error("Expected Polar runtime error, got none")
	}

	o.ClearRules()

	o.LoadString("h(x) if x = 1; h(x) if x.Fake();")
	results, errors = o.QueryRule("h", 1)
	if r := <-results; !reflect.DeepEqual(r, map[string]interface{}{}) {
		t.Error("Expected result, got none")
	}
	if e := <-errors; e == nil {
		t.Error("Expected Polar runtime error, got none")
	}

	results, errors = o.QueryRule("v", 1)
	e := <-errors
	if e == nil {
		t.Error("Expected query for undefined rule to throw error")
	}
	if !strings.Contains(e.Error(), "Query for undefined rule `v`") {
		t.Errorf("Received error does not match expected type: %v", err)
	}
	if r := <-results; r != nil {
		t.Error("Got result; expected none")
	}

}

func TestIsAllowed(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.LoadString("allow(\"foo\", \"bar\", \"baz\");")
	if a, e := o.IsAllowed("foo", "bar", "baz"); e != nil {
		t.Error(e.Error())
	} else if !a {
		t.Error("IsAllowed returned false, expected true")
	}

	if a, e := o.IsAllowed("foo", "baz", "bar"); e != nil {
		t.Error(e.Error())
	} else if a {
		t.Error("IsAllowed returned true, expected false")
	}

}

type User struct {
	Name string
}

type Widget struct {
	Id        int
	CompanyId int
}

type Company struct {
	Id int
}

type Foo struct {
	Name string
	Num  int
}

func MakeFoo(name string, num int) Foo {
	return Foo{name, num}
}

func TestConstructors(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.RegisterClass(reflect.TypeOf(Foo{}), MakeFoo)

	o.LoadString("f(y) if x = new Foo(\"hello\", 123) and y = x.Name;")
	results, errors := o.QueryRule("f", Variable("y"))

	if err = <-errors; err != nil {
		t.Error(err.Error())
	} else {
		var got []map[string]interface{}
		expected := map[string]interface{}{"y": "hello"}
		for elem := range results {
			got = append(got, elem)
		}
		if len(got) != 1 {
			t.Errorf("Received incorrect number of results: %v", got)
		} else if !reflect.DeepEqual(got[0], expected) {
			t.Errorf("Expected: %v, got: %v", expected, got[0])
		}
	}
	y := reflect.TypeOf(nil)
	_ = y
	//o.RegisterClass(reflect.TypeOf(nil), MakeFoo)
}

func TestExpressionError(t *testing.T) {
	var o oso.Oso
	var err error

	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if o.LoadString("f(x) if x > 2;") != nil {
		t.Fatalf("Load string failed: %v", err)
	}

	_, errors := o.QueryRule("f", Variable("x"))
	err = <-errors

	msg := err.Error()

	if !strings.Contains(msg, "unbound") {
		t.Error("Does not contain unbound in error message.")
	}
}

func TestRuleTypes(t *testing.T) {
	var o oso.Oso
	var err error
	var msg string

	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if err = o.RegisterClass(reflect.TypeOf(User{}), nil); err != nil {
		t.Fatalf("Register class failed: %v", err)
	}
	if err = o.RegisterClass(reflect.TypeOf(Widget{}), nil); err != nil {
		t.Fatalf("Register class failed: %v", err)
	}

	policy := "type is_actor(_actor: Actor); is_actor(_actor: Actor);"

	if err = o.LoadString(policy); err != nil {
		t.Fatalf("Load string failed: %v", err)
	}
	if err = o.ClearRules(); err != nil {
		t.Fatalf("Clear rules failed: %v", err)
	}

	policy = "type is_actor(_actor: Actor); is_actor(_actor: Widget);"

	if err = o.LoadString(policy); err == nil {
		t.Fatalf("Failed to raise validation error.")
	} else if msg = err.Error(); !strings.Contains(msg, "Invalid rule") {
		t.Fatalf("Incorrect error message: %v", msg)
	}
}

func TestZeroValueRepr(t *testing.T) {
	ffiPolar := ffi.NewPolarFfi()
	host := host.NewHost(ffiPolar)
	polarValue, err := host.ToPolar(Foo{})
	if err != nil {
		t.Fatalf("host.ToPolar failed: %v", err)
	}
	switch variant := polarValue.ValueVariant.(type) {
	case ValueExternalInstance:
		expected := "oso_test.Foo{Name: Num:0}"
		if *variant.Repr != expected {
			t.Errorf("repr didn't match!\n\tExpected: %v\n\tReceived: %#v", expected, *variant.Repr)
		}
	default:
		t.Fatalf("Expected ValueExternalInstance; received: %v", variant)
	}

	polarValue, err = host.ToPolar(Foo{Name: "Zooey", Num: 42})
	if err != nil {
		t.Fatalf("host.ToPolar failed: %v", err)
	}
	switch variant := polarValue.ValueVariant.(type) {
	case ValueExternalInstance:
		expected := "oso_test.Foo{Name:Zooey Num:42}"
		if *variant.Repr != expected {
			t.Errorf("repr didn't match!\n\tExpected: %v\n\tReceived: %#v", expected, *variant.Repr)
		}
	default:
		t.Fatalf("Expected ValueExternalInstance; received: %v", variant)
	}
}

type Typ struct {
	x int
}

func (t Typ) Method() int {
	return t.x + 1
}

func (t *Typ) PtrMethod() bool {
	return t.x == 1
}

func TestPointerMethods(t *testing.T) {
	var o oso.Oso
	var err error

	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if err = o.RegisterClass(reflect.TypeOf(Typ{}), nil); err != nil {
		t.Fatalf("Register class failed: %v", err)
	}

	typ := Typ{x: 1}
	if typ.Method() != 2 {
		t.Errorf("Bad Method")
	}
	if !typ.PtrMethod() {
		t.Errorf("Bad Method")
	}

	o.LoadString("rule1(x: Typ, y) if y = x.Method(); rule2(x: Typ, y) if y = x.PtrMethod();")

	test := func(rule string, typ interface{}, y_val interface{}) {
		results, errors := o.QueryRule(rule, typ, Variable("y"))
		if err = <-errors; err != nil {
			t.Error(err.Error())
		} else {
			var got []map[string]interface{}
			expected := map[string]interface{}{"y": y_val}
			for elem := range results {
				got = append(got, elem)
			}
			if len(got) > 1 {
				t.Errorf("Received too many results: %v", got)
			} else if !reflect.DeepEqual(got[0], expected) {
				t.Errorf("Expected: %v, got: %v", expected, got[0])
			}
		}
	}

	test("rule1", typ, int64(2))
	test("rule2", typ, true)
}

func TestFailingALot(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.LoadString("f(x) if x.Foo();")

	// Do it 100 times, hoping for bad stuff to happen.
	for i := 0; i < 10000; i++ {
		_, errors := o.QueryStr("f(1)")

		if err = <-errors; err != nil {
			if !strings.Contains(err.Error(), "'1' object has no attribute 'Foo'") {
				t.Errorf("Expected Polar runtime error, got: %s", err)
			}
		} else {
			t.Fatal("oops")
		}
	}
}

func TestPartial(t *testing.T) {
	var o oso.Oso
	var err error

	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if o.LoadString(
		"f(1); "+
			"f(x) if x = 1 and x = 2;"+
			"g(x) if x.bar = 1 and x.baz = 2;") != nil {
		t.Fatalf("Load string failed: %v", err)
	}

	test := func(o oso.Oso, setExpression *bool) error {
		q, err := o.NewQueryFromRule("f", Variable("x"))
		if err != nil {
			t.Fatalf("Failed to construct query: %s", err)
		}
		if setExpression != nil {
			q.SetAcceptExpression(*setExpression)
		}
		first, err := q.Next()
		if err != nil || first == nil {
			t.Errorf("Failed to get result: res: %v, err: %s", first, err)
		}
		if (*first)["x"] != int64(1) {
			t.Errorf("Expected: %v, got: %v", map[string]int{"x": 1}, first)
		}
		second, err := q.Next()
		if err != nil || second != nil {
			t.Errorf("Expected no result, got res: %v, err: %s", second, err)
		}

		q, err = o.NewQueryFromRule("g", Variable("x"))
		if err != nil {
			t.Fatalf("Failed to construct query: %s", err)
		}
		if setExpression != nil {
			q.SetAcceptExpression(*setExpression)
		}
		first, err = q.Next()
		if err != nil {
			return err
		}

		got := (*first)["x"]
		expected := Expression{
			Operator: Operator{OperatorAnd{}},
			Args: []interface{}{
				Expression{
					Operator: Operator{OperatorUnify{}},
					Args: []interface{}{
						int64(1),
						Expression{
							Operator: Operator{OperatorDot{}},
							Args: []interface{}{
								Variable("_this"),
								"bar",
							},
						},
					},
				},
				Expression{
					Operator: Operator{OperatorUnify{}},
					Args: []interface{}{
						int64(2),
						Expression{
							Operator: Operator{OperatorDot{}},
							Args: []interface{}{
								Variable("_this"),
								"baz",
							},
						},
					},
				},
			},
		}

		if !reflect.DeepEqual(got, expected) {
			t.Errorf("Expected: \n%+v,\n got: \n%+v", expected, got)
		}

		return nil
	}

	// Test One: don't call set expression at all
	// expected result: first test passes, second test errors on expression
	var flag bool
	err = test(o, &flag)
	if err == nil {
		t.Errorf("Expected to error on expression")
	} else if !strings.Contains(err.Error(), "Received Expression from Polar VM") {
		t.Errorf("Expected to error on expression, got: %s", err)
	}

	// Test Two: explicitly set expression to false
	// expected result: first test passes, second test errors on expression
	flag = false
	err = test(o, &flag)
	if err == nil {
		t.Errorf("Expected to error on expression")
	} else if !strings.Contains(err.Error(), "Received Expression from Polar VM") {
		t.Errorf("Expected to error on expression, got: %s", err)
	}

	// Test Three: set allow expression to true
	// expected result: first and second tests pass
	flag = true
	err = test(o, &flag)
	if err != nil {
		t.Errorf("Expected to succeed, got: %s", err)
	}
}
