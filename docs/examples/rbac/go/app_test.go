package main

import (
	"testing"

	osoErrors "github.com/osohq/go-oso/errors"
)

func TestPolicy(t *testing.T) {
	oso := SetupOso()

	alphaAssociation := Organization{Name: "Alpha Association"}
	betaBusiness := Organization{Name: "Beta Business"}

	affineTypes := Repository{Name: "Affine Types", Organization: alphaAssociation}
	allocator := Repository{Name: "Allocator", Organization: alphaAssociation}
	bubbleSort := Repository{Name: "Bubble Sort", Organization: betaBusiness}
	benchmarks := Repository{Name: "Benchmarks", Organization: betaBusiness}

	ariana := NewUser("Ariana")
	bhavik := NewUser("Bhavik")

	ariana.AssignRoleForResource("owner", alphaAssociation)
	bhavik.AssignRoleForResource("contributor", bubbleSort)
	bhavik.AssignRoleForResource("maintainer", benchmarks)

	var err error
	if err = oso.Authorize(ariana, "read", affineTypes); err != nil {
		t.Errorf("Expected success; received: %v", err)
	}
	if err = oso.Authorize(ariana, "push", affineTypes); err != nil {
		t.Fatalf("Expected success; received: %v", err)
	}
	if err = oso.Authorize(ariana, "read", allocator); err != nil {
		t.Fatalf("Expected success; received: %v", err)
	}
	if err = oso.Authorize(ariana, "push", allocator); err != nil {
		t.Fatalf("Expected success; received: %v", err)
	}
	err = oso.Authorize(ariana, "read", bubbleSort)
	switch err.(type) {
	default:
		t.Fatalf("Expected NotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	err = oso.Authorize(ariana, "push", bubbleSort)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	err = oso.Authorize(ariana, "read", benchmarks)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	err = oso.Authorize(ariana, "push", benchmarks)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}

	err = oso.Authorize(bhavik, "read", affineTypes)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	err = oso.Authorize(bhavik, "push", affineTypes)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	err = oso.Authorize(bhavik, "read", allocator)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	err = oso.Authorize(bhavik, "push", allocator)
	switch err.(type) {
	default:
		t.Fatalf("ENotFoundError; received: %v", err)
	case *osoErrors.NotFoundError:
	}
	if err = oso.Authorize(bhavik, "read", bubbleSort); err != nil {
		t.Fatalf("Expected success; received: %v", err)
	}
	err = oso.Authorize(bhavik, "push", bubbleSort)
	switch err.(type) {
	default:
		t.Fatalf("Expected ForbiddenError; received: %v", err)
	case *osoErrors.ForbiddenError:
	}
	if err = oso.Authorize(bhavik, "read", benchmarks); err != nil {
		t.Fatalf("Expected success; received: %v", err)
	}
	if err = oso.Authorize(bhavik, "push", benchmarks); err != nil {
		t.Fatalf("Expected success; received: %v", err)
	}
}
