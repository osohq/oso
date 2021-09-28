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

func TestLoadFile(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	if err = o.LoadFile("test.polar"); err != nil {
		t.Error(err.Error())
	}

	if err = o.LoadFile("test.polar"); err == nil {
		t.Error("Failed to error on loading duplicate file")
	}

	if err = o.LoadFile("test.txt"); err == nil {
		t.Error("Failed to error on loading non-polar file (.txt)")
	}

	if err = o.LoadFile("fake.polar"); err == nil {
		t.Error("Failed to error on loading non-existent file")
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

	if err = <-errors; err != nil {
		t.Error(err.Error())
	} else {
		var got []map[string]interface{}
		for elem := range results {
			got = append(got, elem)
		}
		if len(got) > 0 {
			t.Errorf("Received too many results: %v", got)
		}
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
	results, errors := o.QueryRule("f", ValueVariable("x"), ValueVariable("y"))

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
	if r := <-results; r != nil {
		t.Error("Got result; expected none")
	}
	if e := <-errors; e != nil {
		t.Error(e)
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
	Id int
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
	results, errors := o.QueryRule("f", ValueVariable("y"))

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

	_, errors := o.QueryRule("f", ValueVariable("x"))
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
		results, errors := o.QueryRule(rule, typ, ValueVariable("y"))
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
