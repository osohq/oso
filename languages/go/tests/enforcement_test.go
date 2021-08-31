package oso_test

import (
	"reflect"
	"testing"

	oso "github.com/osohq/go-oso"
	"github.com/osohq/go-oso/errors"
)

type Request struct {
	Method string
	Path   string
}

func assertAuthorizationError(err error, isNotFound bool, t *testing.T) {
	if err == nil {
		t.Fatal("Expected forbidden error from Authorize")
	}
	switch err.(type) {
	case *errors.NotFoundError:
		if !isNotFound {
			t.Fatalf("Expected ForbiddenError, got NotFoundError")
		}
	case *errors.ForbiddenError:
		if isNotFound {
			t.Fatalf("Expected NotFoundError, got ForbiddenError")
		}
	default:
		t.Fatalf("Unexpected error from Authorize: %v", err)
	}
}

func getOso(t *testing.T) oso.Oso {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.RegisterClass(reflect.TypeOf(Actor{}), nil)
	o.RegisterClass(reflect.TypeOf(Widget{}), nil)
	o.RegisterClass(reflect.TypeOf(Company{}), nil)
	o.RegisterClass(reflect.TypeOf(Request{}), nil)

	return o
}

func TestAuthorize(t *testing.T) {
	o := getOso(t)
	var err error

	guest := Actor{Name: "guest"}
	admin := Actor{Name: "admin"}
	widget0 := Widget{Id: 0}
	widget1 := Widget{Id: 1}

	o.LoadString("allow(_actor: Actor, \"read\", widget: Widget) if " +
		"widget.Id = 0; " +
		"allow(actor: Actor, \"update\", _widget: Widget) if " +
		"actor.Name = \"admin\";")

	if err = o.Authorize(guest, "read", widget0); err != nil {
		t.Fatalf("Authorize returned error for allowed action: %v", err)
	}
	if err = o.Authorize(admin, "update", widget1); err != nil {
		t.Fatalf("Authorize returned error for allowed action: %v", err)
	}

	// Throws a forbidden error when user can read resource
	err = o.Authorize(guest, "update", widget0)
	assertAuthorizationError(err, false, t)

	// Throws a not found error when user cannot read resource
	err = o.Authorize(guest, "read", widget1)
	assertAuthorizationError(err, true, t)
	err = o.Authorize(guest, "update", widget1)
	assertAuthorizationError(err, true, t)
}

func TestAuthorizeRequest(t *testing.T) {
	o := getOso(t)
	var err error

	guest := Actor{Name: "guest"}
	verified := Actor{Name: "verified"}

	o.LoadString("allow_request(_: Actor{Name: \"guest\"}, request: Request) if " +
		"  request.Path = \"/repos\"; " +
		"allow_request(_: Actor{Name: \"verified\"}, request: Request) if " +
		"  request.Path = \"/account\"; ")

	if err = o.AuthorizeRequest(guest, Request{"GET", "/repos"}); err != nil {
		t.Fatalf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeRequest(guest, Request{"GET", "/other"})
	assertAuthorizationError(err, false, t)

	if err = o.AuthorizeRequest(verified, Request{"GET", "/account"}); err != nil {
		t.Fatalf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeRequest(guest, Request{"GET", "/account"})
	assertAuthorizationError(err, false, t)
}

func TestAuthorizeField(t *testing.T) {
	o := getOso(t)
	var err error

	o.LoadString( // Admins can update all fields
		"allow_field(actor: Actor, \"update\", _widget: Widget, field) if " +
			"  actor.Name = \"admin\" and " +
			"  field in [\"name\", \"purpose\", \"private_field\"]; " +

			// Anybody who can update a field can also read it
			"allow_field(actor, \"read\", widget: Widget, field) if " +
			"  allow_field(actor, \"update\", widget, field); " +

			// Anybody can read public fields
			"allow_field(_: Actor, \"read\", _: Widget, field) if " +
			"  field in [\"name\", \"purpose\"];")

	admin := Actor{"admin"}
	guest := Actor{"guest"}
	widget := Widget{0}

	if err = o.AuthorizeField(admin, "update", widget, "purpose"); err != nil {
		t.Fatalf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeField(admin, "update", widget, "foo")
	assertAuthorizationError(err, false, t)

	if err = o.AuthorizeField(guest, "read", widget, "purpose"); err != nil {
		t.Fatalf("Authorize returned error for allowed action: %v", err)
	}
	err = o.AuthorizeField(guest, "read", widget, "private_field")
	assertAuthorizationError(err, false, t)
}
