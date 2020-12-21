package testing


import (
)


type Call struct {
	Name string
	Args []Value
	Kwargs *map[string]Value
}

type Dictionary struct {
	Fields map[string]Value
}

type ExternalInstance struct {
	InstanceId uint64
	Constructor *Value
	Repr *string
}

type InstanceLiteral struct {
	Tag string
	Fields Dictionary
}

type Node interface {
	isNode()
}

type Node__Rule struct {
	Value Rule
}

func (*Node__Rule) isNode() {}

type Node__Term struct {
	Value Value
}

func (*Node__Term) isNode() {}

type Numeric interface {
	isNumeric()
}

type Numeric__Integer int64

func (*Numeric__Integer) isNumeric() {}

type Numeric__Float float64

func (*Numeric__Float) isNumeric() {}

type Operation struct {
	Operator Operator
	Args []Value
}

type Operator interface {
	isOperator()
}

type Operator__Debug struct {
}

func (*Operator__Debug) isOperator() {}

type Operator__Print struct {
}

func (*Operator__Print) isOperator() {}

type Operator__Cut struct {
}

func (*Operator__Cut) isOperator() {}

type Operator__In struct {
}

func (*Operator__In) isOperator() {}

type Operator__Isa struct {
}

func (*Operator__Isa) isOperator() {}

type Operator__New struct {
}

func (*Operator__New) isOperator() {}

type Operator__Dot struct {
}

func (*Operator__Dot) isOperator() {}

type Operator__Not struct {
}

func (*Operator__Not) isOperator() {}

type Operator__Mul struct {
}

func (*Operator__Mul) isOperator() {}

type Operator__Div struct {
}

func (*Operator__Div) isOperator() {}

type Operator__Mod struct {
}

func (*Operator__Mod) isOperator() {}

type Operator__Rem struct {
}

func (*Operator__Rem) isOperator() {}

type Operator__Add struct {
}

func (*Operator__Add) isOperator() {}

type Operator__Sub struct {
}

func (*Operator__Sub) isOperator() {}

type Operator__Eq struct {
}

func (*Operator__Eq) isOperator() {}

type Operator__Geq struct {
}

func (*Operator__Geq) isOperator() {}

type Operator__Leq struct {
}

func (*Operator__Leq) isOperator() {}

type Operator__Neq struct {
}

func (*Operator__Neq) isOperator() {}

type Operator__Gt struct {
}

func (*Operator__Gt) isOperator() {}

type Operator__Lt struct {
}

func (*Operator__Lt) isOperator() {}

type Operator__Unify struct {
}

func (*Operator__Unify) isOperator() {}

type Operator__Or struct {
}

func (*Operator__Or) isOperator() {}

type Operator__And struct {
}

func (*Operator__And) isOperator() {}

type Operator__ForAll struct {
}

func (*Operator__ForAll) isOperator() {}

type Operator__Assign struct {
}

func (*Operator__Assign) isOperator() {}

type Parameter struct {
	Parameter Value
	Specializer *Value
}

type Partial struct {
	Constraints []Operation
	Variable string
}

type Pattern interface {
	isPattern()
}

type Pattern__Dictionary struct {
	Value Dictionary
}

func (*Pattern__Dictionary) isPattern() {}

type Pattern__Instance struct {
	Value InstanceLiteral
}

func (*Pattern__Instance) isPattern() {}

type QueryEvent interface {
	isQueryEvent()
}

type QueryEvent__None struct {
}

func (*QueryEvent__None) isQueryEvent() {}

type QueryEvent__Done struct {
	Result bool
}

func (*QueryEvent__Done) isQueryEvent() {}

type QueryEvent__Debug struct {
	Message string
}

func (*QueryEvent__Debug) isQueryEvent() {}

type QueryEvent__MakeExternal struct {
	InstanceId uint64
	Constructor Value
}

func (*QueryEvent__MakeExternal) isQueryEvent() {}

type QueryEvent__ExternalCall struct {
	CallId uint64
	Instance Value
	Attribute string
	Args *[]Value
	Kwargs *map[string]Value
}

func (*QueryEvent__ExternalCall) isQueryEvent() {}

type QueryEvent__ExternalIsa struct {
	CallId uint64
	Instance Value
	ClassTag string
}

func (*QueryEvent__ExternalIsa) isQueryEvent() {}

type QueryEvent__ExternalIsSubSpecializer struct {
	CallId uint64
	InstanceId uint64
	LeftClassTag string
	RightClassTag string
}

func (*QueryEvent__ExternalIsSubSpecializer) isQueryEvent() {}

type QueryEvent__ExternalIsSubclass struct {
	CallId uint64
	LeftClassTag string
	RightClassTag string
}

func (*QueryEvent__ExternalIsSubclass) isQueryEvent() {}

type QueryEvent__ExternalUnify struct {
	CallId uint64
	LeftInstanceId uint64
	RightInstanceId uint64
}

func (*QueryEvent__ExternalUnify) isQueryEvent() {}

type QueryEvent__Result struct {
	Bindings map[string]Value
	Trace *TraceResult
}

func (*QueryEvent__Result) isQueryEvent() {}

type QueryEvent__ExternalOp struct {
	CallId uint64
	Operator Operator
	Args []Value
}

func (*QueryEvent__ExternalOp) isQueryEvent() {}

type QueryEvent__NextExternal struct {
	CallId uint64
	Iterable Value
}

func (*QueryEvent__NextExternal) isQueryEvent() {}

type Rule struct {
	Name string
	Params []Parameter
	Body Value
}

type Trace struct {
	Node Node
	Children []Trace
}

type TraceResult struct {
	Trace Trace
	Formatted string
}

type Value interface {
	isValue()
}

type Value__Number struct {
	Value Numeric
}

func (*Value__Number) isValue() {}

type Value__String string

func (*Value__String) isValue() {}

type Value__Boolean bool

func (*Value__Boolean) isValue() {}

type Value__ExternalInstance struct {
	Value ExternalInstance
}

func (*Value__ExternalInstance) isValue() {}

type Value__InstanceLiteral struct {
	Value InstanceLiteral
}

func (*Value__InstanceLiteral) isValue() {}

type Value__Dictionary struct {
	Value Dictionary
}

func (*Value__Dictionary) isValue() {}

type Value__Pattern struct {
	Value Pattern
}

func (*Value__Pattern) isValue() {}

type Value__Call struct {
	Value Call
}

func (*Value__Call) isValue() {}

type Value__List []Value

func (*Value__List) isValue() {}

type Value__Variable string

func (*Value__Variable) isValue() {}

type Value__RestVariable string

func (*Value__RestVariable) isValue() {}

type Value__Expression struct {
	Value Operation
}

func (*Value__Expression) isValue() {}

type Value__Partial struct {
	Value Partial
}

func (*Value__Partial) isValue() {}
