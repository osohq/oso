package oso

import (
	"encoding/json"
	"errors"
	"fmt"
)

// Call struct
type Call struct {
	// Name
	Name string `json:"name"`
	// Args
	Args []Value `json:"args"`
	// Kwargs
	Kwargs *map[string]Value `json:"kwargs"`
}

// Dictionary struct
type Dictionary struct {
	// Fields
	Fields map[string]Value `json:"fields"`
}

// ExternalInstance struct
type ExternalInstance struct {
	// InstanceId
	InstanceId uint64 `json:"instance_id"`
	// Constructor
	Constructor *Value `json:"constructor"`
	// Repr
	Repr *string `json:"repr"`
}

// InstanceLiteral struct
type InstanceLiteral struct {
	// Tag
	Tag string `json:"tag"`
	// Fields
	Fields Dictionary `json:"fields"`
}

// Node enum
type NodeVariant interface {
	isNode()
}

type Node struct {
	NodeVariant
}

func (result *Node) UnmarshalJSON(b []byte) error {
	var rawMap map[string]json.RawMessage

	err := json.Unmarshal(b, &rawMap)
	if err != nil {
		return err
	}

	if len(rawMap) != 1 {
		return errors.New("Deserializing Node as an enum variant; expecting a single key")
	}

	for k, v := range rawMap {
		switch k {

		case "Rule":
			var variant NodeRule
			err := json.Unmarshal(v, &variant)
			*result = Node{&variant}
			return err

		case "Term":
			var variant NodeTerm
			err := json.Unmarshal(v, &variant)
			*result = Node{&variant}
			return err

		default:
			return fmt.Errorf("Unknown variant for Node: %s", k)
		}
	}
	return fmt.Errorf("unreachable")
}

func (variant Node) MarshalJSON() ([]byte, error) {
	switch inner := variant.NodeVariant.(type) {

	case *NodeRule:
		return json.Marshal(map[string]*NodeRule{
			"Rule": inner,
		})

	case *NodeTerm:
		return json.Marshal(map[string]*NodeTerm{
			"Term": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// NodeRule newtype
type NodeRule Rule

func (variant NodeRule) MarshalJSON() ([]byte, error) {
	return json.Marshal(Rule(variant))
}

func (variant *NodeRule) UnmarshalJSON(b []byte) error {
	inner := Rule(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = NodeRule(inner)
	return err
}

func (*NodeRule) isNode() {}

// NodeTerm newtype
type NodeTerm Value

func (variant NodeTerm) MarshalJSON() ([]byte, error) {
	return json.Marshal(Value(variant))
}

func (variant *NodeTerm) UnmarshalJSON(b []byte) error {
	inner := Value(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = NodeTerm(inner)
	return err
}

func (*NodeTerm) isNode() {}

// Numeric enum
type NumericVariant interface {
	isNumeric()
}

type Numeric struct {
	NumericVariant
}

func (result *Numeric) UnmarshalJSON(b []byte) error {
	var rawMap map[string]json.RawMessage

	err := json.Unmarshal(b, &rawMap)
	if err != nil {
		return err
	}

	if len(rawMap) != 1 {
		return errors.New("Deserializing Numeric as an enum variant; expecting a single key")
	}

	for k, v := range rawMap {
		switch k {

		case "Integer":
			var variant NumericInteger
			err := json.Unmarshal(v, &variant)
			*result = Numeric{&variant}
			return err

		case "Float":
			var variant NumericFloat
			err := json.Unmarshal(v, &variant)
			*result = Numeric{&variant}
			return err

		default:
			return fmt.Errorf("Unknown variant for Numeric: %s", k)
		}
	}
	return fmt.Errorf("unreachable")
}

func (variant Numeric) MarshalJSON() ([]byte, error) {
	switch inner := variant.NumericVariant.(type) {

	case *NumericInteger:
		return json.Marshal(map[string]*NumericInteger{
			"Integer": inner,
		})

	case *NumericFloat:
		return json.Marshal(map[string]*NumericFloat{
			"Float": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// NumericInteger newtype
type NumericInteger int64

func (variant NumericInteger) MarshalJSON() ([]byte, error) {
	return json.Marshal(int64(variant))
}

func (variant *NumericInteger) UnmarshalJSON(b []byte) error {
	inner := int64(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = NumericInteger(inner)
	return err
}

func (*NumericInteger) isNumeric() {}

// NumericFloat newtype
type NumericFloat float64

func (variant NumericFloat) MarshalJSON() ([]byte, error) {
	return json.Marshal(float64(variant))
}

func (variant *NumericFloat) UnmarshalJSON(b []byte) error {
	inner := float64(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = NumericFloat(inner)
	return err
}

func (*NumericFloat) isNumeric() {}

// Operation struct
type Operation struct {
	// Operator
	Operator Operator `json:"operator"`
	// Args
	Args []Value `json:"args"`
}

// Operator enum
type OperatorVariant interface {
	isOperator()
}

type Operator struct {
	OperatorVariant
}

func (result *Operator) UnmarshalJSON(b []byte) error {
	var rawMap map[string]json.RawMessage

	err := json.Unmarshal(b, &rawMap)
	if err != nil {
		return err
	}

	if len(rawMap) != 1 {
		return errors.New("Deserializing Operator as an enum variant; expecting a single key")
	}

	for k, v := range rawMap {
		switch k {

		case "Debug":
			var variant OperatorDebug
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Print":
			var variant OperatorPrint
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Cut":
			var variant OperatorCut
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "In":
			var variant OperatorIn
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Isa":
			var variant OperatorIsa
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "New":
			var variant OperatorNew
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Dot":
			var variant OperatorDot
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Not":
			var variant OperatorNot
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Mul":
			var variant OperatorMul
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Div":
			var variant OperatorDiv
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Mod":
			var variant OperatorMod
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Rem":
			var variant OperatorRem
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Add":
			var variant OperatorAdd
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Sub":
			var variant OperatorSub
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Eq":
			var variant OperatorEq
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Geq":
			var variant OperatorGeq
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Leq":
			var variant OperatorLeq
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Neq":
			var variant OperatorNeq
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Gt":
			var variant OperatorGt
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Lt":
			var variant OperatorLt
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Unify":
			var variant OperatorUnify
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Or":
			var variant OperatorOr
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "And":
			var variant OperatorAnd
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "ForAll":
			var variant OperatorForAll
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		case "Assign":
			var variant OperatorAssign
			err := json.Unmarshal(v, &variant)
			*result = Operator{&variant}
			return err

		default:
			return fmt.Errorf("Unknown variant for Operator: %s", k)
		}
	}
	return fmt.Errorf("unreachable")
}

func (variant Operator) MarshalJSON() ([]byte, error) {
	switch inner := variant.OperatorVariant.(type) {

	case *OperatorDebug:
		return json.Marshal(map[string]*OperatorDebug{
			"Debug": inner,
		})

	case *OperatorPrint:
		return json.Marshal(map[string]*OperatorPrint{
			"Print": inner,
		})

	case *OperatorCut:
		return json.Marshal(map[string]*OperatorCut{
			"Cut": inner,
		})

	case *OperatorIn:
		return json.Marshal(map[string]*OperatorIn{
			"In": inner,
		})

	case *OperatorIsa:
		return json.Marshal(map[string]*OperatorIsa{
			"Isa": inner,
		})

	case *OperatorNew:
		return json.Marshal(map[string]*OperatorNew{
			"New": inner,
		})

	case *OperatorDot:
		return json.Marshal(map[string]*OperatorDot{
			"Dot": inner,
		})

	case *OperatorNot:
		return json.Marshal(map[string]*OperatorNot{
			"Not": inner,
		})

	case *OperatorMul:
		return json.Marshal(map[string]*OperatorMul{
			"Mul": inner,
		})

	case *OperatorDiv:
		return json.Marshal(map[string]*OperatorDiv{
			"Div": inner,
		})

	case *OperatorMod:
		return json.Marshal(map[string]*OperatorMod{
			"Mod": inner,
		})

	case *OperatorRem:
		return json.Marshal(map[string]*OperatorRem{
			"Rem": inner,
		})

	case *OperatorAdd:
		return json.Marshal(map[string]*OperatorAdd{
			"Add": inner,
		})

	case *OperatorSub:
		return json.Marshal(map[string]*OperatorSub{
			"Sub": inner,
		})

	case *OperatorEq:
		return json.Marshal(map[string]*OperatorEq{
			"Eq": inner,
		})

	case *OperatorGeq:
		return json.Marshal(map[string]*OperatorGeq{
			"Geq": inner,
		})

	case *OperatorLeq:
		return json.Marshal(map[string]*OperatorLeq{
			"Leq": inner,
		})

	case *OperatorNeq:
		return json.Marshal(map[string]*OperatorNeq{
			"Neq": inner,
		})

	case *OperatorGt:
		return json.Marshal(map[string]*OperatorGt{
			"Gt": inner,
		})

	case *OperatorLt:
		return json.Marshal(map[string]*OperatorLt{
			"Lt": inner,
		})

	case *OperatorUnify:
		return json.Marshal(map[string]*OperatorUnify{
			"Unify": inner,
		})

	case *OperatorOr:
		return json.Marshal(map[string]*OperatorOr{
			"Or": inner,
		})

	case *OperatorAnd:
		return json.Marshal(map[string]*OperatorAnd{
			"And": inner,
		})

	case *OperatorForAll:
		return json.Marshal(map[string]*OperatorForAll{
			"ForAll": inner,
		})

	case *OperatorAssign:
		return json.Marshal(map[string]*OperatorAssign{
			"Assign": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

type OperatorDebug struct{}

func (*OperatorDebug) isOperator() {}

type OperatorPrint struct{}

func (*OperatorPrint) isOperator() {}

type OperatorCut struct{}

func (*OperatorCut) isOperator() {}

type OperatorIn struct{}

func (*OperatorIn) isOperator() {}

type OperatorIsa struct{}

func (*OperatorIsa) isOperator() {}

type OperatorNew struct{}

func (*OperatorNew) isOperator() {}

type OperatorDot struct{}

func (*OperatorDot) isOperator() {}

type OperatorNot struct{}

func (*OperatorNot) isOperator() {}

type OperatorMul struct{}

func (*OperatorMul) isOperator() {}

type OperatorDiv struct{}

func (*OperatorDiv) isOperator() {}

type OperatorMod struct{}

func (*OperatorMod) isOperator() {}

type OperatorRem struct{}

func (*OperatorRem) isOperator() {}

type OperatorAdd struct{}

func (*OperatorAdd) isOperator() {}

type OperatorSub struct{}

func (*OperatorSub) isOperator() {}

type OperatorEq struct{}

func (*OperatorEq) isOperator() {}

type OperatorGeq struct{}

func (*OperatorGeq) isOperator() {}

type OperatorLeq struct{}

func (*OperatorLeq) isOperator() {}

type OperatorNeq struct{}

func (*OperatorNeq) isOperator() {}

type OperatorGt struct{}

func (*OperatorGt) isOperator() {}

type OperatorLt struct{}

func (*OperatorLt) isOperator() {}

type OperatorUnify struct{}

func (*OperatorUnify) isOperator() {}

type OperatorOr struct{}

func (*OperatorOr) isOperator() {}

type OperatorAnd struct{}

func (*OperatorAnd) isOperator() {}

type OperatorForAll struct{}

func (*OperatorForAll) isOperator() {}

type OperatorAssign struct{}

func (*OperatorAssign) isOperator() {}

// Parameter struct
type Parameter struct {
	// Parameter
	Parameter Value `json:"parameter"`
	// Specializer
	Specializer *Value `json:"specializer"`
}

// Partial struct
type Partial struct {
	// Constraints
	Constraints []Operation `json:"constraints"`
	// Variable
	Variable string `json:"variable"`
}

// Pattern enum
type PatternVariant interface {
	isPattern()
}

type Pattern struct {
	PatternVariant
}

func (result *Pattern) UnmarshalJSON(b []byte) error {
	var rawMap map[string]json.RawMessage

	err := json.Unmarshal(b, &rawMap)
	if err != nil {
		return err
	}

	if len(rawMap) != 1 {
		return errors.New("Deserializing Pattern as an enum variant; expecting a single key")
	}

	for k, v := range rawMap {
		switch k {

		case "Dictionary":
			var variant PatternDictionary
			err := json.Unmarshal(v, &variant)
			*result = Pattern{&variant}
			return err

		case "Instance":
			var variant PatternInstance
			err := json.Unmarshal(v, &variant)
			*result = Pattern{&variant}
			return err

		default:
			return fmt.Errorf("Unknown variant for Pattern: %s", k)
		}
	}
	return fmt.Errorf("unreachable")
}

func (variant Pattern) MarshalJSON() ([]byte, error) {
	switch inner := variant.PatternVariant.(type) {

	case *PatternDictionary:
		return json.Marshal(map[string]*PatternDictionary{
			"Dictionary": inner,
		})

	case *PatternInstance:
		return json.Marshal(map[string]*PatternInstance{
			"Instance": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// PatternDictionary newtype
type PatternDictionary Dictionary

func (variant PatternDictionary) MarshalJSON() ([]byte, error) {
	return json.Marshal(Dictionary(variant))
}

func (variant *PatternDictionary) UnmarshalJSON(b []byte) error {
	inner := Dictionary(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = PatternDictionary(inner)
	return err
}

func (*PatternDictionary) isPattern() {}

// PatternInstance newtype
type PatternInstance InstanceLiteral

func (variant PatternInstance) MarshalJSON() ([]byte, error) {
	return json.Marshal(InstanceLiteral(variant))
}

func (variant *PatternInstance) UnmarshalJSON(b []byte) error {
	inner := InstanceLiteral(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = PatternInstance(inner)
	return err
}

func (*PatternInstance) isPattern() {}

// QueryEvent enum
type QueryEventVariant interface {
	isQueryEvent()
}

type QueryEvent struct {
	QueryEventVariant
}

func (result *QueryEvent) UnmarshalJSON(b []byte) error {
	var rawMap map[string]json.RawMessage

	err := json.Unmarshal(b, &rawMap)
	if err != nil {
		return err
	}

	if len(rawMap) != 1 {
		return errors.New("Deserializing QueryEvent as an enum variant; expecting a single key")
	}

	for k, v := range rawMap {
		switch k {

		case "None":
			var variant QueryEventNone
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "Done":
			var variant QueryEventDone
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "Debug":
			var variant QueryEventDebug
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "MakeExternal":
			var variant QueryEventMakeExternal
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "ExternalCall":
			var variant QueryEventExternalCall
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "ExternalIsa":
			var variant QueryEventExternalIsa
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "ExternalIsSubSpecializer":
			var variant QueryEventExternalIsSubSpecializer
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "ExternalIsSubclass":
			var variant QueryEventExternalIsSubclass
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "ExternalUnify":
			var variant QueryEventExternalUnify
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "Result":
			var variant QueryEventResult
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "ExternalOp":
			var variant QueryEventExternalOp
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		case "NextExternal":
			var variant QueryEventNextExternal
			err := json.Unmarshal(v, &variant)
			*result = QueryEvent{&variant}
			return err

		default:
			return fmt.Errorf("Unknown variant for QueryEvent: %s", k)
		}
	}
	return fmt.Errorf("unreachable")
}

func (variant QueryEvent) MarshalJSON() ([]byte, error) {
	switch inner := variant.QueryEventVariant.(type) {

	case *QueryEventNone:
		return json.Marshal(map[string]*QueryEventNone{
			"None": inner,
		})

	case *QueryEventDone:
		return json.Marshal(map[string]*QueryEventDone{
			"Done": inner,
		})

	case *QueryEventDebug:
		return json.Marshal(map[string]*QueryEventDebug{
			"Debug": inner,
		})

	case *QueryEventMakeExternal:
		return json.Marshal(map[string]*QueryEventMakeExternal{
			"MakeExternal": inner,
		})

	case *QueryEventExternalCall:
		return json.Marshal(map[string]*QueryEventExternalCall{
			"ExternalCall": inner,
		})

	case *QueryEventExternalIsa:
		return json.Marshal(map[string]*QueryEventExternalIsa{
			"ExternalIsa": inner,
		})

	case *QueryEventExternalIsSubSpecializer:
		return json.Marshal(map[string]*QueryEventExternalIsSubSpecializer{
			"ExternalIsSubSpecializer": inner,
		})

	case *QueryEventExternalIsSubclass:
		return json.Marshal(map[string]*QueryEventExternalIsSubclass{
			"ExternalIsSubclass": inner,
		})

	case *QueryEventExternalUnify:
		return json.Marshal(map[string]*QueryEventExternalUnify{
			"ExternalUnify": inner,
		})

	case *QueryEventResult:
		return json.Marshal(map[string]*QueryEventResult{
			"Result": inner,
		})

	case *QueryEventExternalOp:
		return json.Marshal(map[string]*QueryEventExternalOp{
			"ExternalOp": inner,
		})

	case *QueryEventNextExternal:
		return json.Marshal(map[string]*QueryEventNextExternal{
			"NextExternal": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

type QueryEventNone struct{}

func (*QueryEventNone) isQueryEvent() {}

// QueryEventDone struct
type QueryEventDone struct {
	// Result
	Result bool `json:"result"`
}

func (*QueryEventDone) isQueryEvent() {}

// QueryEventDebug struct
type QueryEventDebug struct {
	// Message
	Message string `json:"message"`
}

func (*QueryEventDebug) isQueryEvent() {}

// QueryEventMakeExternal struct
type QueryEventMakeExternal struct {
	// InstanceId
	InstanceId uint64 `json:"instance_id"`
	// Constructor
	Constructor Value `json:"constructor"`
}

func (*QueryEventMakeExternal) isQueryEvent() {}

// QueryEventExternalCall struct
type QueryEventExternalCall struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Instance
	Instance Value `json:"instance"`
	// Attribute
	Attribute string `json:"attribute"`
	// Args
	Args *[]Value `json:"args"`
	// Kwargs
	Kwargs *map[string]Value `json:"kwargs"`
}

func (*QueryEventExternalCall) isQueryEvent() {}

// QueryEventExternalIsa struct
type QueryEventExternalIsa struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Instance
	Instance Value `json:"instance"`
	// ClassTag
	ClassTag string `json:"class_tag"`
}

func (*QueryEventExternalIsa) isQueryEvent() {}

// QueryEventExternalIsSubSpecializer struct
type QueryEventExternalIsSubSpecializer struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// InstanceId
	InstanceId uint64 `json:"instance_id"`
	// LeftClassTag
	LeftClassTag string `json:"left_class_tag"`
	// RightClassTag
	RightClassTag string `json:"right_class_tag"`
}

func (*QueryEventExternalIsSubSpecializer) isQueryEvent() {}

// QueryEventExternalIsSubclass struct
type QueryEventExternalIsSubclass struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// LeftClassTag
	LeftClassTag string `json:"left_class_tag"`
	// RightClassTag
	RightClassTag string `json:"right_class_tag"`
}

func (*QueryEventExternalIsSubclass) isQueryEvent() {}

// QueryEventExternalUnify struct
type QueryEventExternalUnify struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// LeftInstanceId
	LeftInstanceId uint64 `json:"left_instance_id"`
	// RightInstanceId
	RightInstanceId uint64 `json:"right_instance_id"`
}

func (*QueryEventExternalUnify) isQueryEvent() {}

// QueryEventResult struct
type QueryEventResult struct {
	// Bindings
	Bindings map[string]Value `json:"bindings"`
	// Trace
	Trace *TraceResult `json:"trace"`
}

func (*QueryEventResult) isQueryEvent() {}

// QueryEventExternalOp struct
type QueryEventExternalOp struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Operator
	Operator Operator `json:"operator"`
	// Args
	Args []Value `json:"args"`
}

func (*QueryEventExternalOp) isQueryEvent() {}

// QueryEventNextExternal struct
type QueryEventNextExternal struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Iterable
	Iterable Value `json:"iterable"`
}

func (*QueryEventNextExternal) isQueryEvent() {}

// Rule struct
type Rule struct {
	// Name
	Name string `json:"name"`
	// Params
	Params []Parameter `json:"params"`
	// Body
	Body Value `json:"body"`
}

// Trace struct
type Trace struct {
	// Node
	Node Node `json:"node"`
	// Children
	Children []Trace `json:"children"`
}

// TraceResult struct
type TraceResult struct {
	// Trace
	Trace Trace `json:"trace"`
	// Formatted
	Formatted string `json:"formatted"`
}

// Value enum
type ValueVariant interface {
	isValue()
}

type Value struct {
	ValueVariant
}

func (result *Value) UnmarshalJSON(b []byte) error {
	var rawMap map[string]json.RawMessage

	err := json.Unmarshal(b, &rawMap)
	if err != nil {
		return err
	}

	if len(rawMap) != 1 {
		return errors.New("Deserializing Value as an enum variant; expecting a single key")
	}

	for k, v := range rawMap {
		switch k {

		case "Number":
			var variant ValueNumber
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "String":
			var variant ValueString
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Boolean":
			var variant ValueBoolean
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "ExternalInstance":
			var variant ValueExternalInstance
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Dictionary":
			var variant ValueDictionary
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Pattern":
			var variant ValuePattern
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Call":
			var variant ValueCall
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "List":
			var variant ValueList
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Variable":
			var variant ValueVariable
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "RestVariable":
			var variant ValueRestVariable
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Expression":
			var variant ValueExpression
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		case "Partial":
			var variant ValuePartial
			err := json.Unmarshal(v, &variant)
			*result = Value{&variant}
			return err

		default:
			return fmt.Errorf("Unknown variant for Value: %s", k)
		}
	}
	return fmt.Errorf("unreachable")
}

func (variant Value) MarshalJSON() ([]byte, error) {
	switch inner := variant.ValueVariant.(type) {

	case *ValueNumber:
		return json.Marshal(map[string]*ValueNumber{
			"Number": inner,
		})

	case *ValueString:
		return json.Marshal(map[string]*ValueString{
			"String": inner,
		})

	case *ValueBoolean:
		return json.Marshal(map[string]*ValueBoolean{
			"Boolean": inner,
		})

	case *ValueExternalInstance:
		return json.Marshal(map[string]*ValueExternalInstance{
			"ExternalInstance": inner,
		})

	case *ValueDictionary:
		return json.Marshal(map[string]*ValueDictionary{
			"Dictionary": inner,
		})

	case *ValuePattern:
		return json.Marshal(map[string]*ValuePattern{
			"Pattern": inner,
		})

	case *ValueCall:
		return json.Marshal(map[string]*ValueCall{
			"Call": inner,
		})

	case *ValueList:
		return json.Marshal(map[string]*ValueList{
			"List": inner,
		})

	case *ValueVariable:
		return json.Marshal(map[string]*ValueVariable{
			"Variable": inner,
		})

	case *ValueRestVariable:
		return json.Marshal(map[string]*ValueRestVariable{
			"RestVariable": inner,
		})

	case *ValueExpression:
		return json.Marshal(map[string]*ValueExpression{
			"Expression": inner,
		})

	case *ValuePartial:
		return json.Marshal(map[string]*ValuePartial{
			"Partial": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// ValueNumber newtype
type ValueNumber Numeric

func (variant ValueNumber) MarshalJSON() ([]byte, error) {
	return json.Marshal(Numeric(variant))
}

func (variant *ValueNumber) UnmarshalJSON(b []byte) error {
	inner := Numeric(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueNumber(inner)
	return err
}

func (*ValueNumber) isValue() {}

// ValueString newtype
type ValueString string

func (variant ValueString) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(variant))
}

func (variant *ValueString) UnmarshalJSON(b []byte) error {
	inner := string(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueString(inner)
	return err
}

func (*ValueString) isValue() {}

// ValueBoolean newtype
type ValueBoolean bool

func (variant ValueBoolean) MarshalJSON() ([]byte, error) {
	return json.Marshal(bool(variant))
}

func (variant *ValueBoolean) UnmarshalJSON(b []byte) error {
	inner := bool(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueBoolean(inner)
	return err
}

func (*ValueBoolean) isValue() {}

// ValueExternalInstance newtype
type ValueExternalInstance ExternalInstance

func (variant ValueExternalInstance) MarshalJSON() ([]byte, error) {
	return json.Marshal(ExternalInstance(variant))
}

func (variant *ValueExternalInstance) UnmarshalJSON(b []byte) error {
	inner := ExternalInstance(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueExternalInstance(inner)
	return err
}

func (*ValueExternalInstance) isValue() {}

// ValueDictionary newtype
type ValueDictionary Dictionary

func (variant ValueDictionary) MarshalJSON() ([]byte, error) {
	return json.Marshal(Dictionary(variant))
}

func (variant *ValueDictionary) UnmarshalJSON(b []byte) error {
	inner := Dictionary(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueDictionary(inner)
	return err
}

func (*ValueDictionary) isValue() {}

// ValuePattern newtype
type ValuePattern Pattern

func (variant ValuePattern) MarshalJSON() ([]byte, error) {
	return json.Marshal(Pattern(variant))
}

func (variant *ValuePattern) UnmarshalJSON(b []byte) error {
	inner := Pattern(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValuePattern(inner)
	return err
}

func (*ValuePattern) isValue() {}

// ValueCall newtype
type ValueCall Call

func (variant ValueCall) MarshalJSON() ([]byte, error) {
	return json.Marshal(Call(variant))
}

func (variant *ValueCall) UnmarshalJSON(b []byte) error {
	inner := Call(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueCall(inner)
	return err
}

func (*ValueCall) isValue() {}

// ValueList newtype
type ValueList []Value

func (variant ValueList) MarshalJSON() ([]byte, error) {
	return json.Marshal([]Value(variant))
}

func (variant *ValueList) UnmarshalJSON(b []byte) error {
	inner := []Value(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueList(inner)
	return err
}

func (*ValueList) isValue() {}

// ValueVariable newtype
type ValueVariable string

func (variant ValueVariable) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(variant))
}

func (variant *ValueVariable) UnmarshalJSON(b []byte) error {
	inner := string(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueVariable(inner)
	return err
}

func (*ValueVariable) isValue() {}

// ValueRestVariable newtype
type ValueRestVariable string

func (variant ValueRestVariable) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(variant))
}

func (variant *ValueRestVariable) UnmarshalJSON(b []byte) error {
	inner := string(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueRestVariable(inner)
	return err
}

func (*ValueRestVariable) isValue() {}

// ValueExpression newtype
type ValueExpression Operation

func (variant ValueExpression) MarshalJSON() ([]byte, error) {
	return json.Marshal(Operation(variant))
}

func (variant *ValueExpression) UnmarshalJSON(b []byte) error {
	inner := Operation(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueExpression(inner)
	return err
}

func (*ValueExpression) isValue() {}

// ValuePartial newtype
type ValuePartial Partial

func (variant ValuePartial) MarshalJSON() ([]byte, error) {
	return json.Marshal(Partial(variant))
}

func (variant *ValuePartial) UnmarshalJSON(b []byte) error {
	inner := Partial(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValuePartial(inner)
	return err
}

func (*ValuePartial) isValue() {}
