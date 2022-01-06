package oso_test

import (
	"fmt"
	"reflect"
	"testing"

	oso "github.com/osohq/go-oso"
	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/types"
)

type Request struct {
	Method string
	Path   string
}

func getOso(t *testing.T) oso.Oso {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.RegisterClassWithNameAndFields(reflect.TypeOf(User{}), nil, "User", map[string]interface{}{
		"Name": "String",
	})
	o.RegisterClassWithNameAndFields(reflect.TypeOf(Widget{}), nil, "Widget", map[string]interface{}{
		"Id": "Integer",
		"Parent": types.Relation{
			Kind:       "one",
			OtherType:  "Company",
			MyField:    "CompanyId",
			OtherField: "Id",
		},
	})
	o.RegisterClassWithNameAndFields(reflect.TypeOf(Company{}), nil, "Company", map[string]interface{}{
		"Id": "Integer",
	})
	o.RegisterClassWithNameAndFields(reflect.TypeOf(Request{}), nil, "Request", map[string]interface{}{
		"Method": "String",
		"Path":   "String",
	})

	return o
}

func assertAuthorizationError(t *testing.T, err error, isNotFound bool) {
	if err == nil {
		t.Fatal("Expected forbidden error from Authorize")
	}
	switch err.(type) {
	case *errors.NotFoundError:
		if !isNotFound {
			t.Error("Expected ForbiddenError, got NotFoundError")
		}
	case *errors.ForbiddenError:
		if isNotFound {
			t.Error("Expected NotFoundError, got ForbiddenError")
		}
	default:
		t.Errorf("Unexpected error from Authorize: %v", err)
	}
}

func assertSetEqual(t *testing.T, results map[interface{}]struct{}, elements []string) {
	if len(results) != len(elements) {
		t.Errorf("Expected %v to contain exactly %v", results, elements)
	}
	for _, element := range elements {
		if _, ok := results[element]; !ok {
			t.Errorf("Expected to find %v in %v", element, results)
		}
	}
}

func TestAuthorize(t *testing.T) {
	o := getOso(t)
	var err error

	guest := User{Name: "guest"}
	admin := User{Name: "admin"}
	widget0 := Widget{Id: 0}
	widget1 := Widget{Id: 1}

	if err = o.LoadString("allow(_actor: User, \"read\", widget: Widget) if " +
		"widget.Id = 0; " +
		"allow(actor: User, \"update\", _widget: Widget) if " +
		"actor.Name = \"admin\";"); err != nil {

		t.Errorf("LoadString returned error: %v", err)
	}

	if err = o.Authorize(guest, "read", widget0); err != nil {
		t.Errorf("Authorize returned error for allowed action: %v", err)
	}
	if err = o.Authorize(admin, "update", widget1); err != nil {
		t.Errorf("Authorize returned error for allowed action: %v", err)
	}

	// Throws a forbidden error when user can read resource
	err = o.Authorize(guest, "update", widget0)
	assertAuthorizationError(t, err, false)

	// Throws a not found error when user cannot read resource
	err = o.Authorize(guest, "read", widget1)
	assertAuthorizationError(t, err, true)
	err = o.Authorize(guest, "update", widget1)
	assertAuthorizationError(t, err, true)
}

func TestAuthorizeRequest(t *testing.T) {
	o := getOso(t)
	var err error

	guest := User{Name: "guest"}
	verified := User{Name: "verified"}

	if err = o.LoadString("allow_request(_: User{Name: \"guest\"}, request: Request) if " +
		"  request.Path = \"/repos\"; " +
		"allow_request(_: User{Name: \"verified\"}, request: Request) if " +
		"  request.Path = \"/account\"; "); err != nil {

		t.Errorf("LoadString returned error: %v", err)
	}

	if err = o.AuthorizeRequest(guest, Request{"GET", "/repos"}); err != nil {
		t.Errorf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeRequest(guest, Request{"GET", "/other"})
	assertAuthorizationError(t, err, false)

	if err = o.AuthorizeRequest(verified, Request{"GET", "/account"}); err != nil {
		t.Errorf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeRequest(guest, Request{"GET", "/account"})
	assertAuthorizationError(t, err, false)
}

func TestAuthorizeField(t *testing.T) {
	o := getOso(t)
	var err error

	o.LoadString( // Admins can update all fields
		"allow_field(actor: User, \"update\", _widget: Widget, field) if " +
			"  actor.Name = \"admin\" and " +
			"  field in [\"name\", \"purpose\", \"private_field\"]; " +

			// Anybody who can update a field can also read it
			"allow_field(actor, \"read\", widget: Widget, field) if " +
			"  allow_field(actor, \"update\", widget, field); " +

			// Anybody can read public fields
			"allow_field(_: User, \"read\", _: Widget, field) if " +
			"  field in [\"name\", \"purpose\"];")

	admin := User{"admin"}
	guest := User{"guest"}
	widget := Widget{0, 0}

	if err = o.AuthorizeField(admin, "update", widget, "purpose"); err != nil {
		t.Errorf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeField(admin, "update", widget, "foo")
	assertAuthorizationError(t, err, false)

	if err = o.AuthorizeField(guest, "read", widget, "purpose"); err != nil {
		t.Errorf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeField(guest, "read", widget, "private_field")
	assertAuthorizationError(t, err, false)
}

func TestAuthorizedActions(t *testing.T) {
	o := getOso(t)
	var err error

	o.LoadString("allow(_actor: User{Name: \"Sally\"}, action, _resource: Widget{Id: 1}) if action in [\"CREATE\", \"READ\"];")

	actor := User{Name: "Sally"}
	resource := Widget{Id: 1}

	res, err := o.AuthorizedActions(actor, resource, false)
	if err != nil {
		t.Fatalf("Failed to get allowed actions: %v", err)
	}
	assertSetEqual(t, res, []string{"CREATE", "READ"})

	o.ClearRules()

	o.LoadString("allow(_actor: User{Name: \"John\"}, _action, _resource: Widget{Id: 1});")

	actor = User{Name: "John"}
	res, err = o.AuthorizedActions(actor, resource, true)
	if err != nil {
		t.Fatalf("Failed to get allowed actions: %v", err)
	}
	if _, ok := res["*"]; !ok {
		t.Error("expected * action")
	}

	_, err = o.AuthorizedActions(actor, resource, false)
	if err == nil {
		t.Fatal("Expected an error from AuthorizedActions")
	}

	res, err = o.AuthorizedActions(actor, Widget{Id: 2}, false)
	if err != nil {
		t.Fatalf("Failed to get allowed actions: %v", err)
	}
	if len(res) != 0 {
		t.Error("expected no actions", res)
	}
}

type TestAdapter struct {
}

func (a TestAdapter) BuildQuery(filter *types.Filter) (interface{}, error) {
	return nil, nil
}

func (a TestAdapter) ExecuteQuery(query interface{}) (interface{}, error) {
	return nil, nil
}

func TestAuthorizedQuery(t *testing.T) {
	o := getOso(t)
	var err error

	o.SetDataFilteringAdapter(&TestAdapter{})

	o.LoadString("allow(_actor: User, \"get\", resource: Widget) if resource.Parent.Id = 1;")

	actor := User{Name: "Sally"}
	resource := Widget{Id: 1}
	_, _ = o.IsAllowed(actor, "get", resource)

	results, err := o.AuthorizedQuery(actor, "get", "Widget")
	fmt.Printf("%v\n", results)
	if err != nil {
		t.Fatalf("Failed to get query: %v", err)
	}
}
func TestAuthorizedFields(t *testing.T) {
	o := getOso(t)
	var res map[interface{}]struct{}

	o.LoadString( // Admins can update all fields
		"allow_field(actor: User, \"update\", _widget: Widget, field) if " +
			"  actor.Name = \"admin\" and " +
			"  field in [\"name\", \"purpose\", \"private_field\"]; " +

			// Anybody who can update a field can also read it
			"allow_field(actor, \"read\", widget: Widget, field) if " +
			"  allow_field(actor, \"update\", widget, field); " +

			// Anybody can read public fields
			"allow_field(_: User, \"read\", _: Widget, field) if " +
			"  field in [\"name\", \"purpose\"];")

	admin := User{"admin"}
	guest := User{"guest"}
	widget := Widget{0, 0}

	// Admins should be able to update all fields
	res, _ = o.AuthorizedFields(admin, "update", widget, false)
	assertSetEqual(t, res, []string{"name", "purpose", "private_field"})
	// Admins should be able to read all fields
	res, _ = o.AuthorizedFields(admin, "read", widget, false)
	assertSetEqual(t, res, []string{"name", "purpose", "private_field"})
	// Guests should not be able to update any fields
	res, _ = o.AuthorizedFields(guest, "update", widget, false)
	assertSetEqual(t, res, []string{})
	// Guests should be able to read public fields
	res, _ = o.AuthorizedFields(guest, "read", widget, false)
	assertSetEqual(t, res, []string{"name", "purpose"})
}

func TestCustomReadAction(t *testing.T) {
	var err error
	o := getOso(t)
	o.SetReadAction("fetch")

	o.LoadString("allow(\"graham\", \"fetch\", \"bar\");")

	err = o.Authorize("sam", "frob", "bar")
	// Should throw not found, sam cannot read bar
	assertAuthorizationError(t, err, true)
	err = o.Authorize("graham", "frob", "bar")
	// Should throw forbidden, graham can read bar
	assertAuthorizationError(t, err, false)
}

type CustomError struct {
	IsNotFound bool
}

func (e *CustomError) Error() string {
	return "CustomError"
}

func TestCustomErrors(t *testing.T) {
	var err error
	o := getOso(t)
	o.SetNotFoundError(func() error { return &CustomError{true} })
	o.SetForbiddenError(func() error { return &CustomError{false} })

	o.LoadString("allow(\"graham\", \"read\", \"bar\");")

	err = o.Authorize("sam", "frob", "bar")
	if custom, ok := err.(*CustomError); ok {
		if !custom.IsNotFound {
			t.Error("Expected CustomError to have IsNotFound = true")
		}
	} else {
		t.Errorf("Expected Authorize to return a CustomError, but got %v", err)
	}

	err = o.Authorize("graham", "frob", "bar")
	if custom, ok := err.(*CustomError); ok {
		if custom.IsNotFound {
			t.Error("Expected CustomError to have IsNotFound = false")
		}
	} else {
		t.Errorf("Expected Authorize to return a CustomError, but got %v", err)
	}
}
