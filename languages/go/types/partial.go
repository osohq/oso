package types

type Expression struct {
	Operator Operator
	Args     []interface{}
}

type Variable string
