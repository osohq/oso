package main

import (
	"log"
	"reflect"

	oso "github.com/osohq/go-oso"
)

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
	Roles map[Role]struct{}
}

func NewUser(name string) User {
	return User{Name: name, Roles: make(map[Role]struct{})}
}

func (u *User) AssignRoleForResource(name string, resource interface{}) {
	u.Roles[Role{Name: name, Resource: resource}] = struct{}{}
	log.Printf("alsdjkflaksjdfklasjdf: %v", u)
}

func SetupOso() oso.Oso {
	o, _ := oso.NewOso()

	o.RegisterClass(reflect.TypeOf(Organization{}), nil)
	o.RegisterClass(reflect.TypeOf(Repository{}), nil)
	o.RegisterClass(reflect.TypeOf(User{}), NewUser)

	o.LoadFiles([]string{"main.polar"})

	return o
}

func main() {
	SetupOso()
}
