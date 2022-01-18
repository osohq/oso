package host

import (
	"reflect"
	"testing"
)

type Role string

const (
	Admin  Role = "ADMIN"
	Member Role = "MEMBER"
)

type Unit struct{}
type Other struct{}

func TestIsInstance(t *testing.T) {
	unit := Unit{}
	other := Other{}
	unitClass := reflect.TypeOf(unit)
	otherClass := reflect.TypeOf(other)

	// positive cases
	if !isInstance(unit, unitClass) {
		t.Error("unit should be an instance of Unit")
	}
	if !isInstance(other, otherClass) {
		t.Error("other should be an instance of Other")
	}
	// negative cases
	if isInstance(unit, otherClass) {
		t.Error("unit should not be an instance of Other")
	}
	if isInstance(other, unitClass) {
		t.Error("other should not be an instance of Unit")
	}

	stringClass := reflect.TypeOf("")
	roleClass := reflect.TypeOf(Admin)
	if !isInstance(Admin, roleClass) || !isInstance(Member, roleClass) {
		t.Error("roles should be instances of the role class")
	}

	// @patrickod #1468 changed Go instance type checking behavior such that
	// NewTypes are no longer considered to be instances of their wrapped
	// underlying type.
	if isInstance(Admin, stringClass) {
		t.Error("Admin is not an instance of String")
	}

}
