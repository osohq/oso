package main

import (
	"reflect"

	oso "github.com/osohq/go-oso"
)

// docs: begin-types
type Organization struct {
	Name string
}

type Repository struct {
	Name         string
	Organization Organization
}

type Role struct {
	Name     string
	Resource interface{}
}

type User struct {
	Name  string
	Roles []Role
}

func NewUser(name string) User {
	return User{Name: name, Roles: []Role{}}
}

func (u *User) AssignRoleForResource(name string, resource interface{}) {
	u.Roles = append(u.Roles, Role{Name: name, Resource: resource})
}

// docs: end-types

func SetupOso() oso.Oso {
// docs: begin-setup
o, _ := oso.NewOso()

// docs: begin-register
o.RegisterClass(reflect.TypeOf(Organization{}), nil)
o.RegisterClass(reflect.TypeOf(Repository{}), nil)
o.RegisterClass(reflect.TypeOf(User{}), NewUser)
// docs: end-register

o.LoadFiles([]string{"main.polar"})
// docs: end-setup

return o
}

func main() {
	SetupOso()
}
