package types

type Expression struct {
	Operator Operator
	Args     []interface{}
}

type Variable string

type TypeFields map[string]Type
type TypeMap map[string]TypeFields
