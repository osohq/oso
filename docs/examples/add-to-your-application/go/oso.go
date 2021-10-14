package main

import (
	"log"
	"reflect"

	"github.com/osohq/go-oso"
)

func initOso() oso.Oso {
	// Initialize the Oso object. This object is usually
	// used globally throughout an application.
	oso, err := oso.NewOso()
	if err != nil {
		log.Fatalf("Failed to set up Oso: %v", err)
	}

	// Tell Oso about the data that you will authorize.
	// These types can be referenced in the policy.
	oso.RegisterClass(reflect.TypeOf(Repository{}), nil)
	oso.RegisterClass(reflect.TypeOf(User{}), nil)

	// Load your policy file.
	if err := oso.LoadFiles([]string{"main.polar"}); err != nil {
		log.Fatalf("Failed to start: %s", err)
	}

	return oso
}
