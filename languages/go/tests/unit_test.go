package oso_test

import (
	"reflect"
	"strings"
	"testing"

	oso "github.com/osohq/go-oso"
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

	o.LoadString("g(x) if x.Fake();")
	results, errors = o.QueryRule("g", 1)

	if err = <-errors; err == nil {
		t.Error("Expected Polar runtime error, got none")
	}

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

type Actor struct {
	Name string
}

type Widget struct {
	Id int
}

type Company struct {
	Id int
}

func TestGetAllowedActions(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.RegisterClass(reflect.TypeOf(Actor{}), nil)
	o.RegisterClass(reflect.TypeOf(Widget{}), nil)
	o.RegisterClass(reflect.TypeOf(Company{}), nil)

	o.LoadString("allow(_actor: Actor{Name: \"Sally\"}, action, _resource: Widget{Id: 1}) if action in [\"CREATE\", \"READ\"];")

	actor := Actor{Name: "Sally"}
	resource := Widget{Id: 1}

	res, err := o.GetAllowedActions(actor, resource, false)
	if err != nil {
		t.Fatalf("Failed to get allowed actions: %v", err)
	}
	if _, ok := res["CREATE"]; !ok {
		t.Error("expected CREATE action")
	}
	if _, ok := res["READ"]; !ok {
		t.Error("expected READ action")
	}

	o.LoadString("allow(_actor: Actor{Name: \"John\"}, _action, _resource: Widget{Id: 1});")

	actor = Actor{Name: "John"}
	res, err = o.GetAllowedActions(actor, resource, true)
	if err != nil {
		t.Fatalf("Failed to get allowed actions: %v", err)
	}
	if _, ok := res["*"]; !ok {
		t.Error("expected * action")
	}

	res, err = o.GetAllowedActions(actor, resource, false)
	if err == nil {
		t.Fatal("Expected an error from GetAllowedActions")
	}

	res, err = o.GetAllowedActions(actor, Widget{Id: 2}, false)
	if err != nil {
		t.Fatalf("Failed to get allowed actions: %v", err)
	}
	if len(res) != 0 {
		t.Error("expected no actions", res)
	}

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
