package oso

import (
	"encoding/json"
	"fmt"
)

// Call comment
type Call struct {
	Name   string
	Args   []Value
	Kwargs *map[string]Value
}

func (c *Call) UnmarshalJSON(b []byte) error {
	var rawData struct {
		Name   string
		Args   []json.RawMessage
		Kwargs *map[string]json.RawMessage
	}
	err := json.Unmarshal(b, &rawData)
	if err != nil {
		return err
	}

	args := make([]Value, len(rawData.Args))
	for i, arg := range rawData.Args {
		val, err := DeserializeValue(arg)
		if err != nil {
			return err
		}
		args[i] = val
	}

	kwargs := make(map[string]Value)
	for k, v := range *rawData.Kwargs {
		val, err := DeserializeValue(v)
		if err != nil {
			return err
		}
		kwargs[k] = val
	}

	*c = Call{
		Name:   rawData.Name,
		Args:   args,
		Kwargs: &kwargs,
	}
	return nil
}

// Dictionary comment
type Dictionary struct {
	Fields map[string]Value
}

// ExternalInstance comment
type ExternalInstance struct {
	InstanceId  uint64
	Constructor *Value
	Repr        *string
}

// InstanceLiteral comment
type InstanceLiteral struct {
	Tag    string
	Fields Dictionary
}

// Node comment
type Node interface {
	isNode()
}

// Node__Rule comment
type Node__Rule struct {
	Value Rule
}

func (*Node__Rule) isNode() {}

// Node__Term comment
type Node__Term struct {
	Value Value
}

func (*Node__Term) isNode() {}

// Numeric comment
type Numeric interface {
	isNumeric()
}

func DeserializeNumeric(b []byte) (Numeric, error) {
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil {
		return nil, err
	}
	for k, v := range raw {
		switch k {
		case "Integer":
			var number Numeric__Integer
			if err := json.Unmarshal(v, &number); err != nil {
				fmt.Printf(string(v))
				return nil, err
			}
			return &number, nil
		case "Float":
			var float Numeric__Integer
			if err := json.Unmarshal(v, &float); err != nil {
				fmt.Printf(string(v))
				return nil, err
			}
			return &float, nil
		default:
			return nil, fmt.Errorf("Unknown variant for Numeric: %s", k)
		}
	}
	return nil, fmt.Errorf("no numeric variant found")
}

// Numeric__Integer comment
type Numeric__Integer int64

func (*Numeric__Integer) isNumeric() {}

// Numeric__Float comment
type Numeric__Float float64

func (*Numeric__Float) isNumeric() {}

// Operation comment
type Operation struct {
	Operator Operator
	Args     []Value
}

// Operator comment
type Operator interface {
	isOperator()
}

// Operator__Debug comment
type Operator__Debug struct {
}

func (*Operator__Debug) isOperator() {}

// Operator__Print comment
type Operator__Print struct {
}

func (*Operator__Print) isOperator() {}

// Operator__Cut comment
type Operator__Cut struct {
}

func (*Operator__Cut) isOperator() {}

// Operator__In comment
type Operator__In struct {
}

func (*Operator__In) isOperator() {}

// Operator__Isa comment
type Operator__Isa struct {
}

func (*Operator__Isa) isOperator() {}

// Operator__New comment
type Operator__New struct {
}

func (*Operator__New) isOperator() {}

// Operator__Dot comment
type Operator__Dot struct {
}

func (*Operator__Dot) isOperator() {}

// Operator__Not comment
type Operator__Not struct {
}

func (*Operator__Not) isOperator() {}

// Operator__Mul comment
type Operator__Mul struct {
}

func (*Operator__Mul) isOperator() {}

// Operator__Div comment
type Operator__Div struct {
}

func (*Operator__Div) isOperator() {}

// Operator__Mod comment
type Operator__Mod struct {
}

func (*Operator__Mod) isOperator() {}

// Operator__Rem comment
type Operator__Rem struct {
}

func (*Operator__Rem) isOperator() {}

// Operator__Add comment
type Operator__Add struct {
}

func (*Operator__Add) isOperator() {}

// Operator__Sub comment
type Operator__Sub struct {
}

func (*Operator__Sub) isOperator() {}

// Operator__Eq comment
type Operator__Eq struct {
}

func (*Operator__Eq) isOperator() {}

// Operator__Geq comment
type Operator__Geq struct {
}

func (*Operator__Geq) isOperator() {}

// Operator__Leq comment
type Operator__Leq struct {
}

func (*Operator__Leq) isOperator() {}

// Operator__Neq comment
type Operator__Neq struct {
}

func (*Operator__Neq) isOperator() {}

// Operator__Gt comment
type Operator__Gt struct {
}

func (*Operator__Gt) isOperator() {}

// Operator__Lt comment
type Operator__Lt struct {
}

func (*Operator__Lt) isOperator() {}

// Operator__Unify comment
type Operator__Unify struct {
}

func (*Operator__Unify) isOperator() {}

// Operator__Or comment
type Operator__Or struct {
}

func (*Operator__Or) isOperator() {}

// Operator__And comment
type Operator__And struct {
}

func (*Operator__And) isOperator() {}

// Operator__ForAll comment
type Operator__ForAll struct {
}

func (*Operator__ForAll) isOperator() {}

// Operator__Assign comment
type Operator__Assign struct {
}

func (*Operator__Assign) isOperator() {}

// Parameter comment
type Parameter struct {
	Parameter   Value
	Specializer *Value
}

// Partial comment
type Partial struct {
	Constraints []Operation
	Variable    string
}

// Pattern comment
type Pattern interface {
	isPattern()
}

// Pattern__Dictionary comment
type Pattern__Dictionary struct {
	Value Dictionary
}

func (*Pattern__Dictionary) isPattern() {}

// Pattern__Instance comment
type Pattern__Instance struct {
	Value InstanceLiteral
}

func (*Pattern__Instance) isPattern() {}

// QueryEvent comment
type QueryEvent interface {
	isQueryEvent()
}

// QueryEvent__None comment
type QueryEvent__None struct {
}

func (*QueryEvent__None) isQueryEvent() {}

// QueryEvent__Done comment
type QueryEvent__Done struct {
	Result bool
}

func (*QueryEvent__Done) isQueryEvent() {}

// QueryEvent__Debug comment
type QueryEvent__Debug struct {
	Message string
}

func (*QueryEvent__Debug) isQueryEvent() {}

// QueryEvent__MakeExternal comment
type QueryEvent__MakeExternal struct {
	InstanceId  uint64
	Constructor Value
}

func (*QueryEvent__MakeExternal) isQueryEvent() {}

// QueryEvent__ExternalCall comment
type QueryEvent__ExternalCall struct {
	CallId    uint64
	Instance  Value
	Attribute string
	Args      *[]Value
	Kwargs    *map[string]Value
}

func (*QueryEvent__ExternalCall) isQueryEvent() {}

// QueryEvent__ExternalIsa comment
type QueryEvent__ExternalIsa struct {
	CallId   uint64
	Instance Value
	ClassTag string
}

func (*QueryEvent__ExternalIsa) isQueryEvent() {}

// QueryEvent__ExternalIsSubSpecializer comment
type QueryEvent__ExternalIsSubSpecializer struct {
	CallId        uint64
	InstanceId    uint64
	LeftClassTag  string
	RightClassTag string
}

func (*QueryEvent__ExternalIsSubSpecializer) isQueryEvent() {}

// QueryEvent__ExternalIsSubclass comment
type QueryEvent__ExternalIsSubclass struct {
	CallId        uint64
	LeftClassTag  string
	RightClassTag string
}

func (*QueryEvent__ExternalIsSubclass) isQueryEvent() {}

// QueryEvent__ExternalUnify comment
type QueryEvent__ExternalUnify struct {
	CallId          uint64
	LeftInstanceId  uint64
	RightInstanceId uint64
}

func (*QueryEvent__ExternalUnify) isQueryEvent() {}

// QueryEvent__Result comment
type QueryEvent__Result struct {
	Bindings map[string]Value
	Trace    *TraceResult
}

func (*QueryEvent__Result) isQueryEvent() {}

// QueryEvent__ExternalOp comment
type QueryEvent__ExternalOp struct {
	CallId   uint64
	Operator Operator
	Args     []Value
}

func (*QueryEvent__ExternalOp) isQueryEvent() {}

// QueryEvent__NextExternal comment
type QueryEvent__NextExternal struct {
	CallId   uint64
	Iterable Value
}

func (*QueryEvent__NextExternal) isQueryEvent() {}

// Rule comment
type Rule struct {
	Name   string
	Params []Parameter
	Body   Value
}

// Trace comment
type Trace struct {
	Node     Node
	Children []Trace
}

// TraceResult comment
type TraceResult struct {
	Trace     Trace
	Formatted string
}

// Value comment
type Value interface {
	isValue()
}

// DeserializeValue comment
func DeserializeValue(b []byte) (Value, error) {
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil {
		return nil, err
	}
	for k, v := range raw {
		switch k {
		case "Number":
			number, err := DeserializeNumeric(v)
			if err != nil {
				return nil, err
			}
			return &Value__Number{Value: number}, nil
		case "Call":
			var call Value__Call
			if err := json.Unmarshal(v, &call); err != nil {
				fmt.Printf(string(v))
				return nil, err
			}
			return &call, nil
		// case 0:
		// 	if val, err := load_Value__Number(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 1:
		// 	if val, err := load_Value__String(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 2:
		// 	if val, err := load_Value__Boolean(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 3:
		// 	if val, err := load_Value__ExternalInstance(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 4:
		// 	if val, err := load_Value__InstanceLiteral(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 5:
		// 	if val, err := load_Value__Dictionary(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 6:
		// 	if val, err := load_Value__Pattern(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 7:
		// 	if val, err := load_Value__Call(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 8:
		// 	if val, err := load_Value__List(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 9:
		// 	if val, err := load_Value__Variable(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 10:
		// 	if val, err := load_Value__RestVariable(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 11:
		// 	if val, err := load_Value__Expression(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		// case 12:
		// 	if val, err := load_Value__Partial(deserializer); err == nil {
		// 		return &val, nil
		// 	} else {
		// 		return nil, err
		// 	}

		default:
			return nil, fmt.Errorf("Unknown variant for Value: %s", k)
		}
	}
	return nil, fmt.Errorf("no value variant found")
}

// Value__Number comment
type Value__Number struct {
	Value Numeric
}

func (v *Value__Number) UnmarshalJSON(b []byte) error {
	var Value Numeric
	err := json.Unmarshal(b, &Value)
	if err != nil {
		fmt.Printf(string(b))
		return err
	}
	*v = Value__Number{Value: Value}
	return nil
}

func (*Value__Number) isValue() {}

// Value__String comment
type Value__String string

func (*Value__String) isValue() {}

// Value__Boolean comment
type Value__Boolean bool

func (*Value__Boolean) isValue() {}

// Value__ExternalInstance comment
type Value__ExternalInstance struct {
	Value ExternalInstance
}

func (*Value__ExternalInstance) isValue() {}

// Value__InstanceLiteral comment
type Value__InstanceLiteral struct {
	Value InstanceLiteral
}

func (*Value__InstanceLiteral) isValue() {}

// Value__Dictionary comment
type Value__Dictionary struct {
	Value Dictionary
}

func (*Value__Dictionary) isValue() {}

// Value__Pattern comment
type Value__Pattern struct {
	Value Pattern
}

func (*Value__Pattern) isValue() {}

// Value__Call comment
type Value__Call struct {
	Value Call
}

func (v *Value__Call) UnmarshalJSON(b []byte) error {
	var Value Call
	err := json.Unmarshal(b, &Value)
	if err != nil {
		return err
	}
	*v = Value__Call{Value: Value}
	return nil
}

func (*Value__Call) isValue() {}

// Value__List comment
type Value__List []Value

func (*Value__List) isValue() {}

// Value__Variable comment
type Value__Variable string

func (*Value__Variable) isValue() {}

// Value__RestVariable comment
type Value__RestVariable string

func (*Value__RestVariable) isValue() {}

// Value__Expression comment
type Value__Expression struct {
	Value Operation
}

func (*Value__Expression) isValue() {}

// Value__Partial comment
type Value__Partial struct {
	Value Partial
}

func (*Value__Partial) isValue() {}
