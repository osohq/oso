package oso_test

import (
	"reflect"
	"testing"

	oso "github.com/osohq/go-oso"
	"github.com/osohq/go-oso/errors"
)

type Actor struct {
	Name string
}

type Widget struct {
	Id int
}

type Company struct {
	Id int
}

func assertAuthorizationError(err error, isNotFound bool, t *testing.T) {
	if err == nil {
		t.Fatal("Expected forbidden error from Authorize")
	}
	if authErr, ok := err.(*errors.AuthorizationError); ok {
		if authErr.IsNotFound != isNotFound {
			t.Fatal("Authorize returned wrong type of AuthorizationError")
		}
	} else {
		t.Fatalf("Unexpected error from Authorize: %v", err)
	}
}

func TestAuthorize(t *testing.T) {
	var o oso.Oso
	var err error
	if o, err = oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}

	o.RegisterClass(reflect.TypeOf(Actor{}), nil)
	o.RegisterClass(reflect.TypeOf(Widget{}), nil)
	o.RegisterClass(reflect.TypeOf(Company{}), nil)

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
