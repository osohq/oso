package types

// Operation struct
type Expression struct {
	// Operator
	Operator Operator
	// Args
	Args []interface{}
}

type Variable string
