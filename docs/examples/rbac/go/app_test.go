package main

import (
	"errors"
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
		t.Errorf("Expected success: %v", err)
	}
	if err = oso.Authorize(ariana, "push", affineTypes); err != nil {
		t.Fatalf("Expected success: %v", err)
	}
	if err = oso.Authorize(ariana, "read", allocator); err != nil {
		t.Fatalf("Expected success: %v", err)
	}
	if err = oso.Authorize(ariana, "push", allocator); err != nil {
		t.Fatalf("Expected success: %v", err)
	}
	if err = oso.Authorize(ariana, "read", bubbleSort); errors.Is(err, &osoErrors.NotFoundError{}) {
		t.Fatalf("Expected failure: %v", err)
	}
	// if err = oso.Authorize(ariana, "push", bubbleSort); errors.Is(err, &osoErrors.NotFoundError{}) {
	// 	t.Fatalf("Expected failure: %v", err)
	// }
	// if err = oso.Authorize(ariana, "read", benchmarks); err != nil {
	// 	t.Fatalf("Expected failure: %v", err)
	// }
	// if err = oso.Authorize(ariana, "push", benchmarks); err != nil {
	// 	t.Fatalf("Expected failure: %v", err)
	// }

	// try { oso.authorize(ariana, "read", bubbleSort); } catch(Exceptions.NotFoundException e) {}
	// try { oso.authorize(ariana, "push", bubbleSort); } catch(Exceptions.NotFoundException e) {}
	// try { oso.authorize(ariana, "read", benchmarks); } catch(Exceptions.NotFoundException e) {}
	// try { oso.authorize(ariana, "push", benchmarks); } catch(Exceptions.NotFoundException e) {}
	//
	// try { oso.authorize(bhavik, "read", affineTypes); } catch(Exceptions.NotFoundException e) {}
	// try { oso.authorize(bhavik, "push", affineTypes); } catch(Exceptions.NotFoundException e) {}
	// try { oso.authorize(bhavik, "read", allocator); } catch(Exceptions.NotFoundException e) {}
	// try { oso.authorize(bhavik, "push", allocator); } catch(Exceptions.NotFoundException e) {}
	// oso.authorize(bhavik, "read", bubbleSort);
	// try { oso.authorize(bhavik, "push", bubbleSort); } catch(Exceptions.ForbiddenException e) {}
	// oso.authorize(bhavik, "read", benchmarks);
	// oso.authorize(bhavik, "push", benchmarks);
}
