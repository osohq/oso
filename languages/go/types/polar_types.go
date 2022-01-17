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
type DeclarationRole struct{}

func (DeclarationRole) isDeclaration() {}

type DeclarationPermission struct{}

func (DeclarationPermission) isDeclaration() {}

// DeclarationRelation newtype
type DeclarationRelation Term

func (variant DeclarationRelation) MarshalJSON() ([]byte, error) {
	return json.Marshal(Term(variant))
}

func (variant *DeclarationRelation) UnmarshalJSON(b []byte) error {
	inner := Term(*variant)
	err := json.Unmarshal(b, &inner)
	*variant = DeclarationRelation(inner)
	return err
}

func (DeclarationRelation) isDeclaration() {}

// Declaration enum
type DeclarationVariant interface {
	isDeclaration()
}

type Declaration struct {
	DeclarationVariant
}

func (result *Declaration) UnmarshalJSON(b []byte) error {
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
			return errors.New("Deserializing Declaration as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "Role":
		var variant DeclarationRole
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Declaration{variant}
		return nil

	case "Permission":
		var variant DeclarationPermission
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Declaration{variant}
		return nil

	case "Relation":
		var variant DeclarationRelation
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = Declaration{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize Declaration: %s", string(b))
}

func (variant Declaration) MarshalJSON() ([]byte, error) {
	switch inner := variant.DeclarationVariant.(type) {

	case DeclarationRole:
		return json.Marshal(map[string]DeclarationRole{
			"Role": inner,
		})

	case DeclarationPermission:
		return json.Marshal(map[string]DeclarationPermission{
			"Permission": inner,
		})

	case DeclarationRelation:
		return json.Marshal(map[string]DeclarationRelation{
			"Relation": inner,
		})

	}

	return nil, fmt.Errorf("unexpected variant of %v", variant)
}

// Dictionary struct
type Dictionary struct {
	// Fields
	Fields map[Symbol]Term `json:"fields"`
}

// ErrorKindParse newtype
type ErrorKindParse ParseErrorKind

func (variant ErrorKindParse) MarshalJSON() ([]byte, error) {
	return json.Marshal(ParseErrorKind(variant))
}

func (variant *ErrorKindParse) UnmarshalJSON(b []byte) error {
	inner := ParseErrorKind(*variant)
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
	// ClassRepr
	ClassRepr *string `json:"class_repr"`
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

// OperationalErrorInvalidState struct
type OperationalErrorInvalidState struct {
	// Msg
	Msg string `json:"msg"`
}

func (OperationalErrorInvalidState) isOperationalError() {}

// OperationalErrorSerialization struct
type OperationalErrorSerialization struct {
	// Msg
	Msg string `json:"msg"`
}

func (OperationalErrorSerialization) isOperationalError() {}

// OperationalErrorUnexpectedValue struct
type OperationalErrorUnexpectedValue struct {
	// Received
	Received Term `json:"received"`
}

func (OperationalErrorUnexpectedValue) isOperationalError() {}

type OperationalErrorUnknown struct{}

func (OperationalErrorUnknown) isOperationalError() {}

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

	case "Serialization":
		var variant OperationalErrorSerialization
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = OperationalError{variant}
		return nil

	case "UnexpectedValue":
		var variant OperationalErrorUnexpectedValue
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

	}

	return fmt.Errorf("Cannot deserialize OperationalError: %s", string(b))
}

func (variant OperationalError) MarshalJSON() ([]byte, error) {
	switch inner := variant.OperationalErrorVariant.(type) {

	case OperationalErrorInvalidState:
		return json.Marshal(map[string]OperationalErrorInvalidState{
			"InvalidState": inner,
		})

	case OperationalErrorSerialization:
		return json.Marshal(map[string]OperationalErrorSerialization{
			"Serialization": inner,
		})

	case OperationalErrorUnexpectedValue:
		return json.Marshal(map[string]OperationalErrorUnexpectedValue{
			"UnexpectedValue": inner,
		})

	case OperationalErrorUnknown:
		return json.Marshal(map[string]OperationalErrorUnknown{
			"Unknown": inner,
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
	switch variant.OperatorVariant.(type) {

	case OperatorDebug:
		return json.Marshal("Debug")

	case OperatorPrint:
		return json.Marshal("Print")

	case OperatorCut:
		return json.Marshal("Cut")

	case OperatorIn:
		return json.Marshal("In")

	case OperatorIsa:
		return json.Marshal("Isa")

	case OperatorNew:
		return json.Marshal("New")

	case OperatorDot:
		return json.Marshal("Dot")

	case OperatorNot:
		return json.Marshal("Not")

	case OperatorMul:
		return json.Marshal("Mul")

	case OperatorDiv:
		return json.Marshal("Div")

	case OperatorMod:
		return json.Marshal("Mod")

	case OperatorRem:
		return json.Marshal("Rem")

	case OperatorAdd:
		return json.Marshal("Add")

	case OperatorSub:
		return json.Marshal("Sub")

	case OperatorEq:
		return json.Marshal("Eq")

	case OperatorGeq:
		return json.Marshal("Geq")

	case OperatorLeq:
		return json.Marshal("Leq")

	case OperatorNeq:
		return json.Marshal("Neq")

	case OperatorGt:
		return json.Marshal("Gt")

	case OperatorLt:
		return json.Marshal("Lt")

	case OperatorUnify:
		return json.Marshal("Unify")

	case OperatorOr:
		return json.Marshal("Or")

	case OperatorAnd:
		return json.Marshal("And")

	case OperatorForAll:
		return json.Marshal("ForAll")

	case OperatorAssign:
		return json.Marshal("Assign")

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

// ParseErrorKindIntegerOverflow struct
type ParseErrorKindIntegerOverflow struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindIntegerOverflow) isParseErrorKind() {}

// ParseErrorKindInvalidTokenCharacter struct
type ParseErrorKindInvalidTokenCharacter struct {
	// Token
	Token string `json:"token"`
	// C
	C string `json:"c"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindInvalidTokenCharacter) isParseErrorKind() {}

// ParseErrorKindInvalidToken struct
type ParseErrorKindInvalidToken struct {
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindInvalidToken) isParseErrorKind() {}

// ParseErrorKindUnrecognizedEOF struct
type ParseErrorKindUnrecognizedEOF struct {
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindUnrecognizedEOF) isParseErrorKind() {}

// ParseErrorKindUnrecognizedToken struct
type ParseErrorKindUnrecognizedToken struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindUnrecognizedToken) isParseErrorKind() {}

// ParseErrorKindExtraToken struct
type ParseErrorKindExtraToken struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindExtraToken) isParseErrorKind() {}

// ParseErrorKindReservedWord struct
type ParseErrorKindReservedWord struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindReservedWord) isParseErrorKind() {}

// ParseErrorKindInvalidFloat struct
type ParseErrorKindInvalidFloat struct {
	// Token
	Token string `json:"token"`
	// Loc
	Loc uint64 `json:"loc"`
}

func (ParseErrorKindInvalidFloat) isParseErrorKind() {}

// ParseErrorKindWrongValueType struct
type ParseErrorKindWrongValueType struct {
	// Loc
	Loc uint64 `json:"loc"`
	// Term
	Term Term `json:"term"`
	// Expected
	Expected string `json:"expected"`
}

func (ParseErrorKindWrongValueType) isParseErrorKind() {}

// ParseErrorKindDuplicateKey struct
type ParseErrorKindDuplicateKey struct {
	// Loc
	Loc uint64 `json:"loc"`
	// Key
	Key string `json:"key"`
}

func (ParseErrorKindDuplicateKey) isParseErrorKind() {}

// ParseErrorKind enum
type ParseErrorKindVariant interface {
	isParseErrorKind()
}

type ParseErrorKind struct {
	ParseErrorKindVariant
}

func (result *ParseErrorKind) UnmarshalJSON(b []byte) error {
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
			return errors.New("Deserializing ParseErrorKind as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}
	switch variantName {

	case "IntegerOverflow":
		var variant ParseErrorKindIntegerOverflow
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "InvalidTokenCharacter":
		var variant ParseErrorKindInvalidTokenCharacter
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "InvalidToken":
		var variant ParseErrorKindInvalidToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "UnrecognizedEOF":
		var variant ParseErrorKindUnrecognizedEOF
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "UnrecognizedToken":
		var variant ParseErrorKindUnrecognizedToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "ExtraToken":
		var variant ParseErrorKindExtraToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "ReservedWord":
		var variant ParseErrorKindReservedWord
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "InvalidFloat":
		var variant ParseErrorKindInvalidFloat
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "WrongValueType":
		var variant ParseErrorKindWrongValueType
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	case "DuplicateKey":
		var variant ParseErrorKindDuplicateKey
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ParseErrorKind{variant}
		return nil

	}

	return fmt.Errorf("Cannot deserialize ParseErrorKind: %s", string(b))
}

func (variant ParseErrorKind) MarshalJSON() ([]byte, error) {
	switch inner := variant.ParseErrorKindVariant.(type) {

	case ParseErrorKindIntegerOverflow:
		return json.Marshal(map[string]ParseErrorKindIntegerOverflow{
			"IntegerOverflow": inner,
		})

	case ParseErrorKindInvalidTokenCharacter:
		return json.Marshal(map[string]ParseErrorKindInvalidTokenCharacter{
			"InvalidTokenCharacter": inner,
		})

	case ParseErrorKindInvalidToken:
		return json.Marshal(map[string]ParseErrorKindInvalidToken{
			"InvalidToken": inner,
		})

	case ParseErrorKindUnrecognizedEOF:
		return json.Marshal(map[string]ParseErrorKindUnrecognizedEOF{
			"UnrecognizedEOF": inner,
		})

	case ParseErrorKindUnrecognizedToken:
		return json.Marshal(map[string]ParseErrorKindUnrecognizedToken{
			"UnrecognizedToken": inner,
		})

	case ParseErrorKindExtraToken:
		return json.Marshal(map[string]ParseErrorKindExtraToken{
			"ExtraToken": inner,
		})

	case ParseErrorKindReservedWord:
		return json.Marshal(map[string]ParseErrorKindReservedWord{
			"ReservedWord": inner,
		})

	case ParseErrorKindInvalidFloat:
		return json.Marshal(map[string]ParseErrorKindInvalidFloat{
			"InvalidFloat": inner,
		})

	case ParseErrorKindWrongValueType:
		return json.Marshal(map[string]ParseErrorKindWrongValueType{
			"WrongValueType": inner,
		})

	case ParseErrorKindDuplicateKey:
		return json.Marshal(map[string]ParseErrorKindDuplicateKey{
			"DuplicateKey": inner,
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

// Rule struct
type Rule struct {
	// Name
	Name Symbol `json:"name"`
	// Params
	Params []Parameter `json:"params"`
	// Body
	Body Term `json:"body"`
	// Required
	Required bool `json:"required"`
}

// RuntimeErrorArithmeticError struct
type RuntimeErrorArithmeticError struct {
	// Term
	Term Term `json:"term"`
}

func (RuntimeErrorArithmeticError) isRuntimeError() {}

// RuntimeErrorUnsupported struct
type RuntimeErrorUnsupported struct {
	// Msg
	Msg string `json:"msg"`
	// Term
	Term Term `json:"term"`
}

func (RuntimeErrorUnsupported) isRuntimeError() {}

// RuntimeErrorTypeError struct
type RuntimeErrorTypeError struct {
	// Msg
	Msg string `json:"msg"`
	// StackTrace
	StackTrace string `json:"stack_trace"`
	// Term
	Term Term `json:"term"`
}

func (RuntimeErrorTypeError) isRuntimeError() {}

// RuntimeErrorStackOverflow struct
type RuntimeErrorStackOverflow struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorStackOverflow) isRuntimeError() {}

// RuntimeErrorQueryTimeout struct
type RuntimeErrorQueryTimeout struct {
	// Elapsed
	Elapsed uint64 `json:"elapsed"`
	// Timeout
	Timeout uint64 `json:"timeout"`
}

func (RuntimeErrorQueryTimeout) isRuntimeError() {}

// RuntimeErrorApplication struct
type RuntimeErrorApplication struct {
	// Msg
	Msg string `json:"msg"`
	// StackTrace
	StackTrace string `json:"stack_trace"`
	// Term
	Term *Term `json:"term"`
}

func (RuntimeErrorApplication) isRuntimeError() {}

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

// RuntimeErrorDataFilteringFieldMissing struct
type RuntimeErrorDataFilteringFieldMissing struct {
	// VarType
	VarType string `json:"var_type"`
	// Field
	Field string `json:"field"`
}

func (RuntimeErrorDataFilteringFieldMissing) isRuntimeError() {}

// RuntimeErrorDataFilteringUnsupportedOp struct
type RuntimeErrorDataFilteringUnsupportedOp struct {
	// Operation
	Operation Operation `json:"operation"`
}

func (RuntimeErrorDataFilteringUnsupportedOp) isRuntimeError() {}

// RuntimeErrorInvalidRegistration struct
type RuntimeErrorInvalidRegistration struct {
	// Sym
	Sym Symbol `json:"sym"`
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorInvalidRegistration) isRuntimeError() {}

type RuntimeErrorMultipleLoadError struct{}

func (RuntimeErrorMultipleLoadError) isRuntimeError() {}

// RuntimeErrorQueryForUndefinedRule struct
type RuntimeErrorQueryForUndefinedRule struct {
	// Name
	Name string `json:"name"`
}

func (RuntimeErrorQueryForUndefinedRule) isRuntimeError() {}

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

	case "DataFilteringFieldMissing":
		var variant RuntimeErrorDataFilteringFieldMissing
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "DataFilteringUnsupportedOp":
		var variant RuntimeErrorDataFilteringUnsupportedOp
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "InvalidRegistration":
		var variant RuntimeErrorInvalidRegistration
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "MultipleLoadError":
		var variant RuntimeErrorMultipleLoadError
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = RuntimeError{variant}
		return nil

	case "QueryForUndefinedRule":
		var variant RuntimeErrorQueryForUndefinedRule
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

	case RuntimeErrorUnsupported:
		return json.Marshal(map[string]RuntimeErrorUnsupported{
			"Unsupported": inner,
		})

	case RuntimeErrorTypeError:
		return json.Marshal(map[string]RuntimeErrorTypeError{
			"TypeError": inner,
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

	case RuntimeErrorIncompatibleBindings:
		return json.Marshal(map[string]RuntimeErrorIncompatibleBindings{
			"IncompatibleBindings": inner,
		})

	case RuntimeErrorUnhandledPartial:
		return json.Marshal(map[string]RuntimeErrorUnhandledPartial{
			"UnhandledPartial": inner,
		})

	case RuntimeErrorDataFilteringFieldMissing:
		return json.Marshal(map[string]RuntimeErrorDataFilteringFieldMissing{
			"DataFilteringFieldMissing": inner,
		})

	case RuntimeErrorDataFilteringUnsupportedOp:
		return json.Marshal(map[string]RuntimeErrorDataFilteringUnsupportedOp{
			"DataFilteringUnsupportedOp": inner,
		})

	case RuntimeErrorInvalidRegistration:
		return json.Marshal(map[string]RuntimeErrorInvalidRegistration{
			"InvalidRegistration": inner,
		})

	case RuntimeErrorMultipleLoadError:
		return json.Marshal(map[string]RuntimeErrorMultipleLoadError{
			"MultipleLoadError": inner,
		})

	case RuntimeErrorQueryForUndefinedRule:
		return json.Marshal(map[string]RuntimeErrorQueryForUndefinedRule{
			"QueryForUndefinedRule": inner,
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

// ValidationErrorFileLoading struct
type ValidationErrorFileLoading struct {
	// Filename
	Filename string `json:"filename"`
	// Contents
	Contents string `json:"contents"`
	// Msg
	Msg string `json:"msg"`
}

func (ValidationErrorFileLoading) isValidationError() {}

// ValidationErrorMissingRequiredRule struct
type ValidationErrorMissingRequiredRule struct {
	// RuleType
	RuleType Rule `json:"rule_type"`
}

func (ValidationErrorMissingRequiredRule) isValidationError() {}

// ValidationErrorInvalidRule struct
type ValidationErrorInvalidRule struct {
	// Rule
	Rule Rule `json:"rule"`
	// Msg
	Msg string `json:"msg"`
}

func (ValidationErrorInvalidRule) isValidationError() {}

// ValidationErrorInvalidRuleType struct
type ValidationErrorInvalidRuleType struct {
	// RuleType
	RuleType Rule `json:"rule_type"`
	// Msg
	Msg string `json:"msg"`
}

func (ValidationErrorInvalidRuleType) isValidationError() {}

// ValidationErrorUndefinedRuleCall struct
type ValidationErrorUndefinedRuleCall struct {
	// Term
	Term Term `json:"term"`
}

func (ValidationErrorUndefinedRuleCall) isValidationError() {}

// ValidationErrorResourceBlock struct
type ValidationErrorResourceBlock struct {
	// Term
	Term Term `json:"term"`
	// Msg
	Msg string `json:"msg"`
}

func (ValidationErrorResourceBlock) isValidationError() {}

// ValidationErrorSingletonVariable struct
type ValidationErrorSingletonVariable struct {
	// Term
	Term Term `json:"term"`
}

func (ValidationErrorSingletonVariable) isValidationError() {}

// ValidationErrorUnregisteredClass struct
type ValidationErrorUnregisteredClass struct {
	// Term
	Term Term `json:"term"`
}

func (ValidationErrorUnregisteredClass) isValidationError() {}

// ValidationErrorDuplicateResourceBlockDeclaration struct
type ValidationErrorDuplicateResourceBlockDeclaration struct {
	// Resource
	Resource Term `json:"resource"`
	// Declaration
	Declaration Term `json:"declaration"`
	// Existing
	Existing Declaration `json:"existing"`
	// New
	New Declaration `json:"new"`
}

func (ValidationErrorDuplicateResourceBlockDeclaration) isValidationError() {}

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

	case "FileLoading":
		var variant ValidationErrorFileLoading
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "MissingRequiredRule":
		var variant ValidationErrorMissingRequiredRule
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

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

	case "UndefinedRuleCall":
		var variant ValidationErrorUndefinedRuleCall
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "ResourceBlock":
		var variant ValidationErrorResourceBlock
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "SingletonVariable":
		var variant ValidationErrorSingletonVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "UnregisteredClass":
		var variant ValidationErrorUnregisteredClass
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}
		*result = ValidationError{variant}
		return nil

	case "DuplicateResourceBlockDeclaration":
		var variant ValidationErrorDuplicateResourceBlockDeclaration
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

	case ValidationErrorFileLoading:
		return json.Marshal(map[string]ValidationErrorFileLoading{
			"FileLoading": inner,
		})

	case ValidationErrorMissingRequiredRule:
		return json.Marshal(map[string]ValidationErrorMissingRequiredRule{
			"MissingRequiredRule": inner,
		})

	case ValidationErrorInvalidRule:
		return json.Marshal(map[string]ValidationErrorInvalidRule{
			"InvalidRule": inner,
		})

	case ValidationErrorInvalidRuleType:
		return json.Marshal(map[string]ValidationErrorInvalidRuleType{
			"InvalidRuleType": inner,
		})

	case ValidationErrorUndefinedRuleCall:
		return json.Marshal(map[string]ValidationErrorUndefinedRuleCall{
			"UndefinedRuleCall": inner,
		})

	case ValidationErrorResourceBlock:
		return json.Marshal(map[string]ValidationErrorResourceBlock{
			"ResourceBlock": inner,
		})

	case ValidationErrorSingletonVariable:
		return json.Marshal(map[string]ValidationErrorSingletonVariable{
			"SingletonVariable": inner,
		})

	case ValidationErrorUnregisteredClass:
		return json.Marshal(map[string]ValidationErrorUnregisteredClass{
			"UnregisteredClass": inner,
		})

	case ValidationErrorDuplicateResourceBlockDeclaration:
		return json.Marshal(map[string]ValidationErrorDuplicateResourceBlockDeclaration{
			"DuplicateResourceBlockDeclaration": inner,
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

type Comparison int

const (
	Eq Comparison = iota
	Neq
	In
)

func (comparison *Comparison) UnmarshalJSON(b []byte) error {
	var cmp string
	err := json.Unmarshal(b, &cmp)
	if err != nil {
		return err
	}
	switch cmp {
	case "Eq":
		*comparison = Eq
	case "Neq":
		*comparison = Neq
	case "In":
		*comparison = In
	}
	return nil
}

type Projection struct {
	TypeName  string
	FieldName string
}

func (proj *Projection) UnmarshalJSON(b []byte) error {
	var l []string
	err := json.Unmarshal(b, &l)
	if err != nil {
		return err
	}
	proj.TypeName = l[0]
	proj.FieldName = l[1]
	return nil
}

type Immediate struct {
	Value interface{}
}

type DatumVariant interface {
	isDatum()
}

type Datum struct {
	DatumVariant
}

func (datum *Datum) UnmarshalJSON(b []byte) error {
	var m map[string]*json.RawMessage
	err := json.Unmarshal(b, &m)
	if err != nil {
		return err
	}
	for k, v := range m {
		switch k {
		case "Immediate":
			var val Value
			err = json.Unmarshal(*v, &val)
			if err != nil {
				return err
			}
			datum.DatumVariant = Immediate{val}
		case "Field":
			var proj Projection
			err = json.Unmarshal(*v, &proj)
			if err != nil {
				return err
			}
			datum.DatumVariant = proj
		}
		break
	}
	return nil
}

func (Projection) isDatum() {}
func (Immediate) isDatum()  {}

type FilterRelation struct {
	FromTypeName  string
	FromFieldName string
	ToTypeName    string
}

func (relation *FilterRelation) UnmarshalJSON(b []byte) error {
	var fields []string
	err := json.Unmarshal(b, &fields)
	if err != nil {
		return err
	}
	relation.FromTypeName = fields[0]
	relation.FromFieldName = fields[1]
	relation.ToTypeName = fields[2]
	return nil
}

type FilterCondition struct {
	Lhs Datum
	Cmp Comparison
	Rhs Datum
}

func (relation *FilterCondition) UnmarshalJSON(b []byte) error {
	var fields []*json.RawMessage

	err := json.Unmarshal(b, &fields)
	if err != nil {
		return err
	}
	var lhs Datum
	err = json.Unmarshal(*fields[0], &lhs)
	if err != nil {
		return err
	}
	var op Comparison
	err = json.Unmarshal(*fields[1], &op)
	if err != nil {
		return err
	}
	var rhs Datum
	err = json.Unmarshal(*fields[2], &rhs)
	if err != nil {
		return err
	}
	relation.Lhs = lhs
	relation.Cmp = op
	relation.Rhs = rhs
	return nil
}

type Filter struct {
	// Root
	Root string `json:"root"`
	// Relations
	Relations []FilterRelation `json:"relations"`
	// Conditions
	Conditions [][]FilterCondition `json:"conditions"`
	// Types
	Types map[string]map[string]interface{}
}
