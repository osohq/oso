package types

import (
	"encoding/json"
	"errors"
	"fmt"
) // Call struct
type Call struct {
	// Name
	Name Symbol `json:"name"`
	// Args
	Args []Term `json:"args"`
	// Kwargs
	Kwargs *map[Symbol]Term `json:"kwargs"`
}

// Dictionary struct
type Dictionary struct {
	// Fields
	Fields map[Symbol]Term `json:"fields"`
}

// ErrorKindParse newtype
type ErrorKindParse ParseError

func (variant ErrorKindParse) MarshalJSON() ([]byte, error) {
	return json.Marshal(ParseError(variant))
}

func (variant *ErrorKindParse) UnmarshalJSON(b []byte) error {
	inner := ParseError(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ErrorKindParse(inner)
	return err
}

func (ErrorKindParse) isErrorKind() {}

// ErrorKindRuntime newtype
type ErrorKindRuntime RuntimeError

func (variant ErrorKindRuntime) MarshalJSON() ([]byte, error) {
	return json.Marshal(RuntimeError(variant))
}

func (variant *ErrorKindRuntime) UnmarshalJSON(b []byte) error {
	inner := RuntimeError(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ErrorKindRuntime(inner)
	return err
}

func (ErrorKindRuntime) isErrorKind() {}

// ErrorKindOperational newtype
type ErrorKindOperational OperationalError

func (variant ErrorKindOperational) MarshalJSON() ([]byte, error) {
	return json.Marshal(OperationalError(variant))
}

func (variant *ErrorKindOperational) UnmarshalJSON(b []byte) error {
	inner := OperationalError(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ErrorKindOperational(inner)
	return err
}

func (ErrorKindOperational) isErrorKind() {}

// ErrorKindParameter newtype
type ErrorKindParameter ParameterError

func (variant ErrorKindParameter) MarshalJSON() ([]byte, error) {
	return json.Marshal(ParameterError(variant))
}

func (variant *ErrorKindParameter) UnmarshalJSON(b []byte) error {
	inner := ParameterError(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ErrorKindParameter(inner)
	return err
}

func (ErrorKindParameter) isErrorKind() {}

// ErrorKindValidation newtype
type ErrorKindValidation ValidationError

func (variant ErrorKindValidation) MarshalJSON() ([]byte, error) {
	return json.Marshal(ValidationError(variant))
}

func (variant *ErrorKindValidation) UnmarshalJSON(b []byte) error {
	inner := ValidationError(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ErrorKindValidation(inner)
	return err
}

func (ErrorKindValidation) isErrorKind() {}

// ErrorKind enum
type ErrorKindVariant interface {
	isErrorKind()
}

type ErrorKind struct {
	ErrorKindVariant
}

func (result *ErrorKind) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing ErrorKind as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Parse":
		var variant ErrorKindParse
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ErrorKind{variant}
		return nil

	case "Runtime":
		var variant ErrorKindRuntime
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ErrorKind{variant}
		return nil

	case "Operational":
		var variant ErrorKindOperational
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ErrorKind{variant}
		return nil

	case "Parameter":
		var variant ErrorKindParameter
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ErrorKind{variant}
		return nil

	case "Validation":
		var variant ErrorKindValidation
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ErrorKind{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize ErrorKind: %s", string(b))
}

func (variant ErrorKind) MarshalJSON() ([]byte, error) {
	switch inner := variant.ErrorKindVariant.(type) {

	case ErrorKindParse:
		return json.Marshal(map[string]ErrorKindParse{
			"Parse": inner,
		})

	case ErrorKindRuntime:
		return json.Marshal(map[string]ErrorKindRuntime{
			"Runtime": inner,
		})

	case ErrorKindOperational:
		return json.Marshal(map[string]ErrorKindOperational{
			"Operational": inner,
		})

	case ErrorKindParameter:
		return json.Marshal(map[string]ErrorKindParameter{
			"Parameter": inner,
		})

	case ErrorKindValidation:
		return json.Marshal(map[string]ErrorKindValidation{
			"Validation": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// ExternalInstance struct
type ExternalInstance struct {
	// InstanceId
	InstanceId uint64 `json:"instance_id"`
	// Constructor
	Constructor *Term `json:"constructor"`
	// Repr
	Repr *string `json:"repr"`
}

// FormattedPolarError struct
type FormattedPolarError struct {
	// Kind
	Kind ErrorKind `json:"kind"`
	// Formatted
	Formatted string `json:"formatted"`
}

// InstanceLiteral struct
type InstanceLiteral struct {
	// Tag
	Tag Symbol `json:"tag"`
	// Fields
	Fields Dictionary `json:"fields"`
}

// Message struct
type Message struct {
	// Kind
	Kind MessageKind `json:"kind"`
	// Msg
	Msg string `json:"msg"`
}
type MessageKindPrint struct{}

func (MessageKindPrint) isMessageKind() {}

type MessageKindWarning struct{}

func (MessageKindWarning) isMessageKind() {}

// MessageKind enum
type MessageKindVariant interface {
	isMessageKind()
}

type MessageKind struct {
	MessageKindVariant
}

func (result *MessageKind) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing MessageKind as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Print":
		var variant MessageKindPrint
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = MessageKind{variant}
		return nil

	case "Warning":
		var variant MessageKindWarning
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = MessageKind{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize MessageKind: %s", string(b))
}

func (variant MessageKind) MarshalJSON() ([]byte, error) {
	switch inner := variant.MessageKindVariant.(type) {

	case MessageKindPrint:
		return json.Marshal(map[string]MessageKindPrint{
			"Print": inner,
		})

	case MessageKindWarning:
		return json.Marshal(map[string]MessageKindWarning{
			"Warning": inner,
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

func (NodeRule) isNode() {}

// NodeTerm newtype
type NodeTerm Term

func (variant NodeTerm) MarshalJSON() ([]byte, error) {
	return json.Marshal(Term(variant))
}

func (variant *NodeTerm) UnmarshalJSON(b []byte) error {
	inner := Term(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = NodeTerm(inner)
	return err
}

func (NodeTerm) isNode() {}

// Node enum
type NodeVariant interface {
	isNode()
}

type Node struct {
	NodeVariant
}

func (result *Node) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing Node as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Rule":
		var variant NodeRule
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Node{variant}
		return nil

	case "Term":
		var variant NodeTerm
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Node{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize Node: %s", string(b))
}

func (variant Node) MarshalJSON() ([]byte, error) {
	switch inner := variant.NodeVariant.(type) {

	case NodeRule:
		return json.Marshal(map[string]NodeRule{
			"Rule": inner,
		})

	case NodeTerm:
		return json.Marshal(map[string]NodeTerm{
			"Term": inner,
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

func (NumericInteger) isNumeric() {}

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

func (NumericFloat) isNumeric() {}

// Numeric enum
type NumericVariant interface {
	isNumeric()
}

type Numeric struct {
	NumericVariant
}

func (result *Numeric) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing Numeric as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Integer":
		var variant NumericInteger
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Numeric{variant}
		return nil

	case "Float":
		var variant NumericFloat
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Numeric{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize Numeric: %s", string(b))
}

func (variant Numeric) MarshalJSON() ([]byte, error) {
	switch inner := variant.NumericVariant.(type) {

	case NumericInteger:
		return json.Marshal(map[string]NumericInteger{
			"Integer": inner,
		})

	case NumericFloat:
		return json.Marshal(map[string]NumericFloat{
			"Float": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// Operation struct
type Operation struct {
	// Operator
	Operator Operator `json:"operator"`
	// Args
	Args []Term `json:"args"`
}

// OperationalErrorUnimplemented struct
type OperationalErrorUnimplemented struct {
	// Msg
	Msg string `json:"msg"`
}

func (OperationalErrorUnimplemented) isOperationalError() {}

type OperationalErrorUnknown struct{}

func (OperationalErrorUnknown) isOperationalError() {}

// OperationalErrorInvalidState struct
type OperationalErrorInvalidState struct {
	// Msg
	Msg string `json:"msg"`
}

func (OperationalErrorInvalidState) isOperationalError() {}

// OperationalError enum
type OperationalErrorVariant interface {
	isOperationalError()
}

type OperationalError struct {
	OperationalErrorVariant
}

func (result *OperationalError) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing OperationalError as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Unimplemented":
		var variant OperationalErrorUnimplemented
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = OperationalError{variant}
		return nil

	case "Unknown":
		var variant OperationalErrorUnknown
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = OperationalError{variant}
		return nil

	case "InvalidState":
		var variant OperationalErrorInvalidState
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = OperationalError{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize OperationalError: %s", string(b))
}

func (variant OperationalError) MarshalJSON() ([]byte, error) {
	switch inner := variant.OperationalErrorVariant.(type) {

	case OperationalErrorUnimplemented:
		return json.Marshal(map[string]OperationalErrorUnimplemented{
			"Unimplemented": inner,
		})

	case OperationalErrorUnknown:
		return json.Marshal(map[string]OperationalErrorUnknown{
			"Unknown": inner,
		})

	case OperationalErrorInvalidState:
		return json.Marshal(map[string]OperationalErrorInvalidState{
			"InvalidState": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

type OperatorDebug struct{}

func (OperatorDebug) isOperator() {}

type OperatorPrint struct{}

func (OperatorPrint) isOperator() {}

type OperatorCut struct{}

func (OperatorCut) isOperator() {}

type OperatorIn struct{}

func (OperatorIn) isOperator() {}

type OperatorIsa struct{}

func (OperatorIsa) isOperator() {}

type OperatorNew struct{}

func (OperatorNew) isOperator() {}

type OperatorDot struct{}

func (OperatorDot) isOperator() {}

type OperatorNot struct{}

func (OperatorNot) isOperator() {}

type OperatorMul struct{}

func (OperatorMul) isOperator() {}

type OperatorDiv struct{}

func (OperatorDiv) isOperator() {}

type OperatorMod struct{}

func (OperatorMod) isOperator() {}

type OperatorRem struct{}

func (OperatorRem) isOperator() {}

type OperatorAdd struct{}

func (OperatorAdd) isOperator() {}

type OperatorSub struct{}

func (OperatorSub) isOperator() {}

type OperatorEq struct{}

func (OperatorEq) isOperator() {}

type OperatorGeq struct{}

func (OperatorGeq) isOperator() {}

type OperatorLeq struct{}

func (OperatorLeq) isOperator() {}

type OperatorNeq struct{}

func (OperatorNeq) isOperator() {}

type OperatorGt struct{}

func (OperatorGt) isOperator() {}

type OperatorLt struct{}

func (OperatorLt) isOperator() {}

type OperatorUnify struct{}

func (OperatorUnify) isOperator() {}

type OperatorOr struct{}

func (OperatorOr) isOperator() {}

type OperatorAnd struct{}

func (OperatorAnd) isOperator() {}

type OperatorForAll struct{}

func (OperatorForAll) isOperator() {}

type OperatorAssign struct{}

func (OperatorAssign) isOperator() {}

// Operator enum
type OperatorVariant interface {
	isOperator()
}

type Operator struct {
	OperatorVariant
}

func (result *Operator) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing Operator as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Debug":
		var variant OperatorDebug
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Print":
		var variant OperatorPrint
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Cut":
		var variant OperatorCut
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "In":
		var variant OperatorIn
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Isa":
		var variant OperatorIsa
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "New":
		var variant OperatorNew
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Dot":
		var variant OperatorDot
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Not":
		var variant OperatorNot
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Mul":
		var variant OperatorMul
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Div":
		var variant OperatorDiv
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Mod":
		var variant OperatorMod
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Rem":
		var variant OperatorRem
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Add":
		var variant OperatorAdd
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Sub":
		var variant OperatorSub
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Eq":
		var variant OperatorEq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Geq":
		var variant OperatorGeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Leq":
		var variant OperatorLeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Neq":
		var variant OperatorNeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Gt":
		var variant OperatorGt
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Lt":
		var variant OperatorLt
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Unify":
		var variant OperatorUnify
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Or":
		var variant OperatorOr
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "And":
		var variant OperatorAnd
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "ForAll":
		var variant OperatorForAll
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	case "Assign":
		var variant OperatorAssign
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Operator{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize Operator: %s", string(b))
}

func (variant Operator) MarshalJSON() ([]byte, error) {
	switch inner := variant.OperatorVariant.(type) {

	case OperatorDebug:
		return json.Marshal(map[string]OperatorDebug{
			"Debug": inner,
		})

	case OperatorPrint:
		return json.Marshal(map[string]OperatorPrint{
			"Print": inner,
		})

	case OperatorCut:
		return json.Marshal(map[string]OperatorCut{
			"Cut": inner,
		})

	case OperatorIn:
		return json.Marshal(map[string]OperatorIn{
			"In": inner,
		})

	case OperatorIsa:
		return json.Marshal(map[string]OperatorIsa{
			"Isa": inner,
		})

	case OperatorNew:
		return json.Marshal(map[string]OperatorNew{
			"New": inner,
		})

	case OperatorDot:
		return json.Marshal(map[string]OperatorDot{
			"Dot": inner,
		})

	case OperatorNot:
		return json.Marshal(map[string]OperatorNot{
			"Not": inner,
		})

	case OperatorMul:
		return json.Marshal(map[string]OperatorMul{
			"Mul": inner,
		})

	case OperatorDiv:
		return json.Marshal(map[string]OperatorDiv{
			"Div": inner,
		})

	case OperatorMod:
		return json.Marshal(map[string]OperatorMod{
			"Mod": inner,
		})

	case OperatorRem:
		return json.Marshal(map[string]OperatorRem{
			"Rem": inner,
		})

	case OperatorAdd:
		return json.Marshal(map[string]OperatorAdd{
			"Add": inner,
		})

	case OperatorSub:
		return json.Marshal(map[string]OperatorSub{
			"Sub": inner,
		})

	case OperatorEq:
		return json.Marshal(map[string]OperatorEq{
			"Eq": inner,
		})

	case OperatorGeq:
		return json.Marshal(map[string]OperatorGeq{
			"Geq": inner,
		})

	case OperatorLeq:
		return json.Marshal(map[string]OperatorLeq{
			"Leq": inner,
		})

	case OperatorNeq:
		return json.Marshal(map[string]OperatorNeq{
			"Neq": inner,
		})

	case OperatorGt:
		return json.Marshal(map[string]OperatorGt{
			"Gt": inner,
		})

	case OperatorLt:
		return json.Marshal(map[string]OperatorLt{
			"Lt": inner,
		})

	case OperatorUnify:
		return json.Marshal(map[string]OperatorUnify{
			"Unify": inner,
		})

	case OperatorOr:
		return json.Marshal(map[string]OperatorOr{
			"Or": inner,
		})

	case OperatorAnd:
		return json.Marshal(map[string]OperatorAnd{
			"And": inner,
		})

	case OperatorForAll:
		return json.Marshal(map[string]OperatorForAll{
			"ForAll": inner,
		})

	case OperatorAssign:
		return json.Marshal(map[string]OperatorAssign{
			"Assign": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// Parameter struct
type Parameter struct {
	// Parameter
	Parameter Term `json:"parameter"`
	// Specializer
	Specializer *Term `json:"specializer"`
}

// ParameterError newtype
type ParameterError string

func (variant ParameterError) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(variant))
}

func (variant *ParameterError) UnmarshalJSON(b []byte) error {
	inner := string(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ParameterError(inner)
	return err
}

// ParseErrorIntegerOverflow struct
type ParseErrorIntegerOverflow struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorIntegerOverflow) isParseError() {}

// ParseErrorInvalidTokenCharacter struct
type ParseErrorInvalidTokenCharacter struct {
	// Token
	Token string `json:"token"`
	// C
	C string `json:"c"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorInvalidTokenCharacter) isParseError() {}

// ParseErrorInvalidToken struct
type ParseErrorInvalidToken struct {
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorInvalidToken) isParseError() {}

// ParseErrorUnrecognizedEOF struct
type ParseErrorUnrecognizedEOF struct {
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorUnrecognizedEOF) isParseError() {}

// ParseErrorUnrecognizedToken struct
type ParseErrorUnrecognizedToken struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorUnrecognizedToken) isParseError() {}

// ParseErrorExtraToken struct
type ParseErrorExtraToken struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorExtraToken) isParseError() {}

// ParseErrorReservedWord struct
type ParseErrorReservedWord struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorReservedWord) isParseError() {}

// ParseErrorInvalidFloat struct
type ParseErrorInvalidFloat struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorInvalidFloat) isParseError() {}

// ParseErrorWrongValueType struct
type ParseErrorWrongValueType struct {
	// Loc
	Loc uint64 `json:"loc"`
	// Term
	Term Term `json:"term"`
	// Expected
	Expected string `json:"expected"`
}

func (ParseErrorWrongValueType) isParseError() {}

// ParseErrorDuplicateKey struct
type ParseErrorDuplicateKey struct {
	// Loc
	Loc uint64 `json:"loc"`
	// Key
	Key string `json:"key"`
}

func (ParseErrorDuplicateKey) isParseError() {}

// ParseErrorSingletonVariable struct
type ParseErrorSingletonVariable struct {
	// Loc
	Loc uint64 `json:"loc"`
	// Name
	Name string `json:"name"`
}

func (ParseErrorSingletonVariable) isParseError() {}

// ParseErrorResourceBlock struct
type ParseErrorResourceBlock struct {
	// Loc
	Loc uint64 `json:"loc"`
	// Msg
	Msg string `json:"msg"`
	// Ranges
	Ranges []Range `json:"ranges"`
}

func (ParseErrorResourceBlock) isParseError() {}

// ParseError enum
type ParseErrorVariant interface {
	isParseError()
}

type ParseError struct {
	ParseErrorVariant
}

func (result *ParseError) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing ParseError as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "IntegerOverflow":
		var variant ParseErrorIntegerOverflow
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "InvalidTokenCharacter":
		var variant ParseErrorInvalidTokenCharacter
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "InvalidToken":
		var variant ParseErrorInvalidToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "UnrecognizedEOF":
		var variant ParseErrorUnrecognizedEOF
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "UnrecognizedToken":
		var variant ParseErrorUnrecognizedToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "ExtraToken":
		var variant ParseErrorExtraToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "ReservedWord":
		var variant ParseErrorReservedWord
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "InvalidFloat":
		var variant ParseErrorInvalidFloat
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "WrongValueType":
		var variant ParseErrorWrongValueType
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "DuplicateKey":
		var variant ParseErrorDuplicateKey
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "SingletonVariable":
		var variant ParseErrorSingletonVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	case "ResourceBlock":
		var variant ParseErrorResourceBlock
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseError{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize ParseError: %s", string(b))
}

func (variant ParseError) MarshalJSON() ([]byte, error) {
	switch inner := variant.ParseErrorVariant.(type) {

	case ParseErrorIntegerOverflow:
		return json.Marshal(map[string]ParseErrorIntegerOverflow{
			"IntegerOverflow": inner,
		})

	case ParseErrorInvalidTokenCharacter:
		return json.Marshal(map[string]ParseErrorInvalidTokenCharacter{
			"InvalidTokenCharacter": inner,
		})

	case ParseErrorInvalidToken:
		return json.Marshal(map[string]ParseErrorInvalidToken{
			"InvalidToken": inner,
		})

	case ParseErrorUnrecognizedEOF:
		return json.Marshal(map[string]ParseErrorUnrecognizedEOF{
			"UnrecognizedEOF": inner,
		})

	case ParseErrorUnrecognizedToken:
		return json.Marshal(map[string]ParseErrorUnrecognizedToken{
			"UnrecognizedToken": inner,
		})

	case ParseErrorExtraToken:
		return json.Marshal(map[string]ParseErrorExtraToken{
			"ExtraToken": inner,
		})

	case ParseErrorReservedWord:
		return json.Marshal(map[string]ParseErrorReservedWord{
			"ReservedWord": inner,
		})

	case ParseErrorInvalidFloat:
		return json.Marshal(map[string]ParseErrorInvalidFloat{
			"InvalidFloat": inner,
		})

	case ParseErrorWrongValueType:
		return json.Marshal(map[string]ParseErrorWrongValueType{
			"WrongValueType": inner,
		})

	case ParseErrorDuplicateKey:
		return json.Marshal(map[string]ParseErrorDuplicateKey{
			"DuplicateKey": inner,
		})

	case ParseErrorSingletonVariable:
		return json.Marshal(map[string]ParseErrorSingletonVariable{
			"SingletonVariable": inner,
		})

	case ParseErrorResourceBlock:
		return json.Marshal(map[string]ParseErrorResourceBlock{
			"ResourceBlock": inner,
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

func (PatternDictionary) isPattern() {}

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

func (PatternInstance) isPattern() {}

// Pattern enum
type PatternVariant interface {
	isPattern()
}

type Pattern struct {
	PatternVariant
}

func (result *Pattern) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing Pattern as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Dictionary":
		var variant PatternDictionary
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Pattern{variant}
		return nil

	case "Instance":
		var variant PatternInstance
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Pattern{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize Pattern: %s", string(b))
}

func (variant Pattern) MarshalJSON() ([]byte, error) {
	switch inner := variant.PatternVariant.(type) {

	case PatternDictionary:
		return json.Marshal(map[string]PatternDictionary{
			"Dictionary": inner,
		})

	case PatternInstance:
		return json.Marshal(map[string]PatternInstance{
			"Instance": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

type QueryEventNone struct{}

func (QueryEventNone) isQueryEvent() {}

// QueryEventDone struct
type QueryEventDone struct {
	// Result
	Result bool `json:"result"`
}

func (QueryEventDone) isQueryEvent() {}

// QueryEventDebug struct
type QueryEventDebug struct {
	// Message
	Message string `json:"message"`
}

func (QueryEventDebug) isQueryEvent() {}

// QueryEventMakeExternal struct
type QueryEventMakeExternal struct {
	// InstanceId
	InstanceId uint64 `json:"instance_id"`
	// Constructor
	Constructor Term `json:"constructor"`
}

func (QueryEventMakeExternal) isQueryEvent() {}

// QueryEventExternalCall struct
type QueryEventExternalCall struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Instance
	Instance Term `json:"instance"`
	// Attribute
	Attribute Symbol `json:"attribute"`
	// Args
	Args *[]Term `json:"args"`
	// Kwargs
	Kwargs *map[Symbol]Term `json:"kwargs"`
}

func (QueryEventExternalCall) isQueryEvent() {}

// QueryEventExternalIsa struct
type QueryEventExternalIsa struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Instance
	Instance Term `json:"instance"`
	// ClassTag
	ClassTag Symbol `json:"class_tag"`
}

func (QueryEventExternalIsa) isQueryEvent() {}

// QueryEventExternalIsaWithPath struct
type QueryEventExternalIsaWithPath struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// BaseTag
	BaseTag Symbol `json:"base_tag"`
	// Path
	Path []Term `json:"path"`
	// ClassTag
	ClassTag Symbol `json:"class_tag"`
}

func (QueryEventExternalIsaWithPath) isQueryEvent() {}

// QueryEventExternalIsSubSpecializer struct
type QueryEventExternalIsSubSpecializer struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// InstanceId
	InstanceId uint64 `json:"instance_id"`
	// LeftClassTag
	LeftClassTag Symbol `json:"left_class_tag"`
	// RightClassTag
	RightClassTag Symbol `json:"right_class_tag"`
}

func (QueryEventExternalIsSubSpecializer) isQueryEvent() {}

// QueryEventExternalIsSubclass struct
type QueryEventExternalIsSubclass struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// LeftClassTag
	LeftClassTag Symbol `json:"left_class_tag"`
	// RightClassTag
	RightClassTag Symbol `json:"right_class_tag"`
}

func (QueryEventExternalIsSubclass) isQueryEvent() {}

// QueryEventResult struct
type QueryEventResult struct {
	// Bindings
	Bindings map[Symbol]Term `json:"bindings"`
	// Trace
	Trace *TraceResult `json:"trace"`
}

func (QueryEventResult) isQueryEvent() {}

// QueryEventExternalOp struct
type QueryEventExternalOp struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Operator
	Operator Operator `json:"operator"`
	// Args
	Args []Term `json:"args"`
}

func (QueryEventExternalOp) isQueryEvent() {}

// QueryEventNextExternal struct
type QueryEventNextExternal struct {
	// CallId
	CallId uint64 `json:"call_id"`
	// Iterable
	Iterable Term `json:"iterable"`
}

func (QueryEventNextExternal) isQueryEvent() {}

// QueryEvent enum
type QueryEventVariant interface {
	isQueryEvent()
}

type QueryEvent struct {
	QueryEventVariant
}

func (result *QueryEvent) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing QueryEvent as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "None":
		var variant QueryEventNone
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "Done":
		var variant QueryEventDone
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "Debug":
		var variant QueryEventDebug
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "MakeExternal":
		var variant QueryEventMakeExternal
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "ExternalCall":
		var variant QueryEventExternalCall
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "ExternalIsa":
		var variant QueryEventExternalIsa
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "ExternalIsaWithPath":
		var variant QueryEventExternalIsaWithPath
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "ExternalIsSubSpecializer":
		var variant QueryEventExternalIsSubSpecializer
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "ExternalIsSubclass":
		var variant QueryEventExternalIsSubclass
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "Result":
		var variant QueryEventResult
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "ExternalOp":
		var variant QueryEventExternalOp
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	case "NextExternal":
		var variant QueryEventNextExternal
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = QueryEvent{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize QueryEvent: %s", string(b))
}

func (variant QueryEvent) MarshalJSON() ([]byte, error) {
	switch inner := variant.QueryEventVariant.(type) {

	case QueryEventNone:
		return json.Marshal(map[string]QueryEventNone{
			"None": inner,
		})

	case QueryEventDone:
		return json.Marshal(map[string]QueryEventDone{
			"Done": inner,
		})

	case QueryEventDebug:
		return json.Marshal(map[string]QueryEventDebug{
			"Debug": inner,
		})

	case QueryEventMakeExternal:
		return json.Marshal(map[string]QueryEventMakeExternal{
			"MakeExternal": inner,
		})

	case QueryEventExternalCall:
		return json.Marshal(map[string]QueryEventExternalCall{
			"ExternalCall": inner,
		})

	case QueryEventExternalIsa:
		return json.Marshal(map[string]QueryEventExternalIsa{
			"ExternalIsa": inner,
		})

	case QueryEventExternalIsaWithPath:
		return json.Marshal(map[string]QueryEventExternalIsaWithPath{
			"ExternalIsaWithPath": inner,
		})

	case QueryEventExternalIsSubSpecializer:
		return json.Marshal(map[string]QueryEventExternalIsSubSpecializer{
			"ExternalIsSubSpecializer": inner,
		})

	case QueryEventExternalIsSubclass:
		return json.Marshal(map[string]QueryEventExternalIsSubclass{
			"ExternalIsSubclass": inner,
		})

	case QueryEventResult:
		return json.Marshal(map[string]QueryEventResult{
			"Result": inner,
		})

	case QueryEventExternalOp:
		return json.Marshal(map[string]QueryEventExternalOp{
			"ExternalOp": inner,
		})

	case QueryEventNextExternal:
		return json.Marshal(map[string]QueryEventNextExternal{
			"NextExternal": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// Range struct
type Range struct {
	// Start
	Start uint64 `json:"start"`
	// End
	End uint64 `json:"end"`
}

// Rule struct
type Rule struct {
	// Name
	Name Symbol `json:"name"`
	// Params
	Params []Parameter `json:"params"`
	// Body
	Body Term `json:"body"`
}

// RuntimeErrorArithmeticError struct
type RuntimeErrorArithmeticError struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorArithmeticError) isRuntimeError() {}

// RuntimeErrorSerialization struct
type RuntimeErrorSerialization struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorSerialization) isRuntimeError() {}

// RuntimeErrorUnsupported struct
type RuntimeErrorUnsupported struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorUnsupported) isRuntimeError() {}

// RuntimeErrorTypeError struct
type RuntimeErrorTypeError struct {
	// Msg
	Msg string `json:"msg"`
	// StackTrace
	StackTrace *string `json:"stack_trace"`
}

func (RuntimeErrorTypeError) isRuntimeError() {}

// RuntimeErrorUnboundVariable struct
type RuntimeErrorUnboundVariable struct {
	// Sym
	Sym Symbol `json:"sym"`
}

func (RuntimeErrorUnboundVariable) isRuntimeError() {}

// RuntimeErrorStackOverflow struct
type RuntimeErrorStackOverflow struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorStackOverflow) isRuntimeError() {}

// RuntimeErrorQueryTimeout struct
type RuntimeErrorQueryTimeout struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorQueryTimeout) isRuntimeError() {}

// RuntimeErrorApplication struct
type RuntimeErrorApplication struct {
	// Msg
	Msg string `json:"msg"`
	// StackTrace
	StackTrace *string `json:"stack_trace"`
}

func (RuntimeErrorApplication) isRuntimeError() {}

// RuntimeErrorFileLoading struct
type RuntimeErrorFileLoading struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorFileLoading) isRuntimeError() {}

// RuntimeErrorIncompatibleBindings struct
type RuntimeErrorIncompatibleBindings struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorIncompatibleBindings) isRuntimeError() {}

// RuntimeErrorUnhandledPartial struct
type RuntimeErrorUnhandledPartial struct {
	// Var
	Var Symbol `json:"var"`
	// Term
	Term Term `json:"term"`
}

func (RuntimeErrorUnhandledPartial) isRuntimeError() {}

// RuntimeError enum
type RuntimeErrorVariant interface {
	isRuntimeError()
}

type RuntimeError struct {
	RuntimeErrorVariant
}

func (result *RuntimeError) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing RuntimeError as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "ArithmeticError":
		var variant RuntimeErrorArithmeticError
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "Serialization":
		var variant RuntimeErrorSerialization
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "Unsupported":
		var variant RuntimeErrorUnsupported
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "TypeError":
		var variant RuntimeErrorTypeError
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "UnboundVariable":
		var variant RuntimeErrorUnboundVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "StackOverflow":
		var variant RuntimeErrorStackOverflow
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "QueryTimeout":
		var variant RuntimeErrorQueryTimeout
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "Application":
		var variant RuntimeErrorApplication
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "FileLoading":
		var variant RuntimeErrorFileLoading
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "IncompatibleBindings":
		var variant RuntimeErrorIncompatibleBindings
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "UnhandledPartial":
		var variant RuntimeErrorUnhandledPartial
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize RuntimeError: %s", string(b))
}

func (variant RuntimeError) MarshalJSON() ([]byte, error) {
	switch inner := variant.RuntimeErrorVariant.(type) {

	case RuntimeErrorArithmeticError:
		return json.Marshal(map[string]RuntimeErrorArithmeticError{
			"ArithmeticError": inner,
		})

	case RuntimeErrorSerialization:
		return json.Marshal(map[string]RuntimeErrorSerialization{
			"Serialization": inner,
		})

	case RuntimeErrorUnsupported:
		return json.Marshal(map[string]RuntimeErrorUnsupported{
			"Unsupported": inner,
		})

	case RuntimeErrorTypeError:
		return json.Marshal(map[string]RuntimeErrorTypeError{
			"TypeError": inner,
		})

	case RuntimeErrorUnboundVariable:
		return json.Marshal(map[string]RuntimeErrorUnboundVariable{
			"UnboundVariable": inner,
		})

	case RuntimeErrorStackOverflow:
		return json.Marshal(map[string]RuntimeErrorStackOverflow{
			"StackOverflow": inner,
		})

	case RuntimeErrorQueryTimeout:
		return json.Marshal(map[string]RuntimeErrorQueryTimeout{
			"QueryTimeout": inner,
		})

	case RuntimeErrorApplication:
		return json.Marshal(map[string]RuntimeErrorApplication{
			"Application": inner,
		})

	case RuntimeErrorFileLoading:
		return json.Marshal(map[string]RuntimeErrorFileLoading{
			"FileLoading": inner,
		})

	case RuntimeErrorIncompatibleBindings:
		return json.Marshal(map[string]RuntimeErrorIncompatibleBindings{
			"IncompatibleBindings": inner,
		})

	case RuntimeErrorUnhandledPartial:
		return json.Marshal(map[string]RuntimeErrorUnhandledPartial{
			"UnhandledPartial": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// Source struct
type Source struct {
	// Filename
	Filename *string `json:"filename"`
	// Src
	Src string `json:"src"`
}

// Symbol newtype
type Symbol string

func (variant Symbol) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(variant))
}

func (variant *Symbol) UnmarshalJSON(b []byte) error {
	inner := string(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = Symbol(inner)
	return err
}

// Term struct
type Term struct {
	// Value
	Value Value `json:"value"`
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

// ValidationErrorInvalidRule struct
type ValidationErrorInvalidRule struct {
	// Rule
	Rule string `json:"rule"`
	// Msg
	Msg string `json:"msg"`
}

func (ValidationErrorInvalidRule) isValidationError() {}

// ValidationErrorInvalidRuleType struct
type ValidationErrorInvalidRuleType struct {
	// RuleType
	RuleType string `json:"rule_type"`
	// Msg
	Msg string `json:"msg"`
}

func (ValidationErrorInvalidRuleType) isValidationError() {}

// ValidationErrorUndefinedRule struct
type ValidationErrorUndefinedRule struct {
	// RuleName
	RuleName string `json:"rule_name"`
}

func (ValidationErrorUndefinedRule) isValidationError() {}

// ValidationError enum
type ValidationErrorVariant interface {
	isValidationError()
}

type ValidationError struct {
	ValidationErrorVariant
}

func (result *ValidationError) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing ValidationError as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "InvalidRule":
		var variant ValidationErrorInvalidRule
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "InvalidRuleType":
		var variant ValidationErrorInvalidRuleType
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "UndefinedRule":
		var variant ValidationErrorUndefinedRule
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize ValidationError: %s", string(b))
}

func (variant ValidationError) MarshalJSON() ([]byte, error) {
	switch inner := variant.ValidationErrorVariant.(type) {

	case ValidationErrorInvalidRule:
		return json.Marshal(map[string]ValidationErrorInvalidRule{
			"InvalidRule": inner,
		})

	case ValidationErrorInvalidRuleType:
		return json.Marshal(map[string]ValidationErrorInvalidRuleType{
			"InvalidRuleType": inner,
		})

	case ValidationErrorUndefinedRule:
		return json.Marshal(map[string]ValidationErrorUndefinedRule{
			"UndefinedRule": inner,
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

func (ValueNumber) isValue() {}

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

func (ValueString) isValue() {}

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

func (ValueBoolean) isValue() {}

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

func (ValueExternalInstance) isValue() {}

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

func (ValueDictionary) isValue() {}

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

func (ValuePattern) isValue() {}

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

func (ValueCall) isValue() {}

// ValueList newtype
type ValueList []Term

func (variant ValueList) MarshalJSON() ([]byte, error) {
	return json.Marshal([]Term(variant))
}

func (variant *ValueList) UnmarshalJSON(b []byte) error {
	inner := []Term(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueList(inner)
	return err
}

func (ValueList) isValue() {}

// ValueVariable newtype
type ValueVariable Symbol

func (variant ValueVariable) MarshalJSON() ([]byte, error) {
	return json.Marshal(Symbol(variant))
}

func (variant *ValueVariable) UnmarshalJSON(b []byte) error {
	inner := Symbol(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueVariable(inner)
	return err
}

func (ValueVariable) isValue() {}

// ValueRestVariable newtype
type ValueRestVariable Symbol

func (variant ValueRestVariable) MarshalJSON() ([]byte, error) {
	return json.Marshal(Symbol(variant))
}

func (variant *ValueRestVariable) UnmarshalJSON(b []byte) error {
	inner := Symbol(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = ValueRestVariable(inner)
	return err
}

func (ValueRestVariable) isValue() {}

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

func (ValueExpression) isValue() {}

// Value enum
type ValueVariant interface {
	isValue()
}

type Value struct {
	ValueVariant
}

func (result *Value) UnmarshalJSON(b []byte) error {
	var variantName string
	var variantValue *json.RawMessage

	// try and deserialize as a string first
	err := json.Unmarshal(b, &variantName)
	if err != nil {
		var rawMap map[string]json.RawMessage
		err := json.Unmarshal(b, &rawMap)
		if err != nil {
			return err
		}
		// JSON should be of form {"VariantName": {...}}
		if len(rawMap) != 1 {
			return errors.New("Deserializing Value as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Number":
		var variant ValueNumber
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "String":
		var variant ValueString
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "Boolean":
		var variant ValueBoolean
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "ExternalInstance":
		var variant ValueExternalInstance
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "Dictionary":
		var variant ValueDictionary
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "Pattern":
		var variant ValuePattern
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "Call":
		var variant ValueCall
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "List":
		var variant ValueList
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "Variable":
		var variant ValueVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "RestVariable":
		var variant ValueRestVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	case "Expression":
		var variant ValueExpression
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Value{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize Value: %s", string(b))
}

func (variant Value) MarshalJSON() ([]byte, error) {
	switch inner := variant.ValueVariant.(type) {

	case ValueNumber:
		return json.Marshal(map[string]ValueNumber{
			"Number": inner,
		})

	case ValueString:
		return json.Marshal(map[string]ValueString{
			"String": inner,
		})

	case ValueBoolean:
		return json.Marshal(map[string]ValueBoolean{
			"Boolean": inner,
		})

	case ValueExternalInstance:
		return json.Marshal(map[string]ValueExternalInstance{
			"ExternalInstance": inner,
		})

	case ValueDictionary:
		return json.Marshal(map[string]ValueDictionary{
			"Dictionary": inner,
		})

	case ValuePattern:
		return json.Marshal(map[string]ValuePattern{
			"Pattern": inner,
		})

	case ValueCall:
		return json.Marshal(map[string]ValueCall{
			"Call": inner,
		})

	case ValueList:
		return json.Marshal(map[string]ValueList{
			"List": inner,
		})

	case ValueVariable:
		return json.Marshal(map[string]ValueVariable{
			"Variable": inner,
		})

	case ValueRestVariable:
		return json.Marshal(map[string]ValueRestVariable{
			"RestVariable": inner,
		})

	case ValueExpression:
		return json.Marshal(map[string]ValueExpression{
			"Expression": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}
