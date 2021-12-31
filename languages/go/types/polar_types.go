package types

import (
	"encoding/json"
	"errors"
	"fmt"
)

// Call struct
type Call struct {
	// Name
	Name Symbol `json:"name"`
	// Args
	Args []Term `json:"args"`
	// Kwargs
	Kwargs *map[Symbol]Term `json:"kwargs"`
}

type ComparisonEq struct{}

func (ComparisonEq) isComparison() {}

type ComparisonNeq struct{}

func (ComparisonNeq) isComparison() {}

type ComparisonIn struct{}

func (ComparisonIn) isComparison() {}

type ComparisonNin struct{}

func (ComparisonNin) isComparison() {}

type ComparisonLt struct{}

func (ComparisonLt) isComparison() {}

type ComparisonLeq struct{}

func (ComparisonLeq) isComparison() {}

type ComparisonGt struct{}

func (ComparisonGt) isComparison() {}

type ComparisonGeq struct{}

func (ComparisonGeq) isComparison() {}

// Comparison enum
//
// The Rust enum type Comparison is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Comparison as a possibility for Comparison.
//
// To make this clear, we prefix all variants with Comparison
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Comparison. Instead, you
// _must_ call DeserializeComparison.
type Comparison interface {
	isComparison()
}

type ComparisonDeserializer struct {
	Comparison
}

func DeserializeComparison(b []byte) (*Comparison, error) {
	var deserializer ComparisonDeserializer
	var result Comparison
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Comparison
	return &result, nil
}

func (result *ComparisonDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Comparison as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}

	switch variantName {
	case "Eq":
		var variant ComparisonEq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "Neq":
		var variant ComparisonNeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "In":
		var variant ComparisonIn
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "Nin":
		var variant ComparisonNin
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "Lt":
		var variant ComparisonLt
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "Leq":
		var variant ComparisonLeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "Gt":
		var variant ComparisonGt
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	case "Geq":
		var variant ComparisonGeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ComparisonDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Comparison: %s", string(b))
}

func (v ComparisonDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeComparison(v.Comparison)
}

func SerializeComparison(variant Comparison) ([]byte, error) {
	switch inner := variant.(type) {
	case ComparisonEq:
		return json.Marshal("Eq")
	case ComparisonNeq:
		return json.Marshal("Neq")
	case ComparisonIn:
		return json.Marshal("In")
	case ComparisonNin:
		return json.Marshal("Nin")
	case ComparisonLt:
		return json.Marshal("Lt")
	case ComparisonLeq:
		return json.Marshal("Leq")
	case ComparisonGt:
		return json.Marshal("Gt")
	case ComparisonGeq:
		return json.Marshal("Geq")
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Condition struct
//
// This mimics the Rust tuple type structure by constructing a
// field for each index in the tuple.
type Condition struct {
	V0 Datum
	V1 Comparison
	V2 Datum
}

func (result *Condition) UnmarshalJSON(b []byte) error {
	var jsonFields []json.RawMessage
	json.Unmarshal(b, &jsonFields)

	if len(jsonFields) != 3 {
		return fmt.Errorf("incorrect length for tuple. Expected %d, got %#v", 3, jsonFields)
	}

	var err error
	v0, err := DeserializeDatum(jsonFields[0])
	if err != nil {
		return err
	}
	result.V0 = *v0
	v1, err := DeserializeComparison(jsonFields[1])
	if err != nil {
		return err
	}
	result.V1 = *v1
	v2, err := DeserializeDatum(jsonFields[2])
	if err != nil {
		return err
	}
	result.V2 = *v2
	return nil
}

func (variant Condition) MarshalJSON() ([]byte, error) {
	fieldArray := []interface{}{
		variant.V0,
		variant.V1,
		variant.V2,
	}

	return json.Marshal(fieldArray)
}

// Constraint struct
type Constraint struct {
	// Kind
	Kind ConstraintKind `json:"kind"`
	// Field
	Field *string `json:"field"`
	// Value
	Value ConstraintValue `json:"value"`
}

func (result *Constraint) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawConstraint struct {
		// Kind
		Kind ConstraintKindDeserializer `json:"kind"`
		// Field
		Field *string `json:"field"`
		// Value
		Value ConstraintValueDeserializer `json:"value"`
	}
	var intermediate RawConstraint
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = Constraint{
		Kind:  intermediate.Kind.ConstraintKind,
		Field: intermediate.Field,
		Value: intermediate.Value.ConstraintValue,
	}
	return nil
}

func (v Constraint) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawConstraint struct {
		// Kind
		Kind ConstraintKindDeserializer `json:"kind"`
		// Field
		Field *string `json:"field"`
		// Value
		Value ConstraintValueDeserializer `json:"value"`
	}
	intermediate := RawConstraint{
		Kind:  ConstraintKindDeserializer{ConstraintKind: v.Kind},
		Field: v.Field,
		Value: ConstraintValueDeserializer{ConstraintValue: v.Value},
	}
	return json.Marshal(intermediate)
}

type ConstraintKindEq struct{}

func (ConstraintKindEq) isConstraintKind() {}

type ConstraintKindIn struct{}

func (ConstraintKindIn) isConstraintKind() {}

type ConstraintKindContains struct{}

func (ConstraintKindContains) isConstraintKind() {}

type ConstraintKindNeq struct{}

func (ConstraintKindNeq) isConstraintKind() {}

type ConstraintKindNin struct{}

func (ConstraintKindNin) isConstraintKind() {}

// ConstraintKind enum
//
// The Rust enum type ConstraintKind is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of ConstraintKind as a possibility for ConstraintKind.
//
// To make this clear, we prefix all variants with ConstraintKind
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of ConstraintKind. Instead, you
// _must_ call DeserializeConstraintKind.
type ConstraintKind interface {
	isConstraintKind()
}

type ConstraintKindDeserializer struct {
	ConstraintKind
}

func DeserializeConstraintKind(b []byte) (*ConstraintKind, error) {
	var deserializer ConstraintKindDeserializer
	var result ConstraintKind
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.ConstraintKind
	return &result, nil
}

func (result *ConstraintKindDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing ConstraintKind as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}

	switch variantName {
	case "Eq":
		var variant ConstraintKindEq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintKindDeserializer{variant}
		return nil
	case "In":
		var variant ConstraintKindIn
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintKindDeserializer{variant}
		return nil
	case "Contains":
		var variant ConstraintKindContains
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintKindDeserializer{variant}
		return nil
	case "Neq":
		var variant ConstraintKindNeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintKindDeserializer{variant}
		return nil
	case "Nin":
		var variant ConstraintKindNin
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintKindDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize ConstraintKind: %s", string(b))
}

func (v ConstraintKindDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeConstraintKind(v.ConstraintKind)
}

func SerializeConstraintKind(variant ConstraintKind) ([]byte, error) {
	switch inner := variant.(type) {
	case ConstraintKindEq:
		return json.Marshal("Eq")
	case ConstraintKindIn:
		return json.Marshal("In")
	case ConstraintKindContains:
		return json.Marshal("Contains")
	case ConstraintKindNeq:
		return json.Marshal("Neq")
	case ConstraintKindNin:
		return json.Marshal("Nin")
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

type ConstraintValueTerm Term

func (variant ConstraintValueTerm) MarshalJSON() ([]byte, error) {
	return json.Marshal((Term)(variant))
}

func (result *ConstraintValueTerm) UnmarshalJSON(b []byte) error {
	var inner Term
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ConstraintValueTerm(inner)
	return nil
}

func (ConstraintValueTerm) isConstraintValue() {}

type ConstraintValueRef Ref

func (variant ConstraintValueRef) MarshalJSON() ([]byte, error) {
	return json.Marshal((Ref)(variant))
}

func (result *ConstraintValueRef) UnmarshalJSON(b []byte) error {
	var inner Ref
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ConstraintValueRef(inner)
	return nil
}

func (ConstraintValueRef) isConstraintValue() {}

type ConstraintValueField string

func (variant ConstraintValueField) MarshalJSON() ([]byte, error) {
	return json.Marshal((string)(variant))
}

func (result *ConstraintValueField) UnmarshalJSON(b []byte) error {
	var inner string
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ConstraintValueField(inner)
	return nil
}

func (ConstraintValueField) isConstraintValue() {}

// ConstraintValue enum
//
// The Rust enum type ConstraintValue is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of ConstraintValue as a possibility for ConstraintValue.
//
// To make this clear, we prefix all variants with ConstraintValue
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of ConstraintValue. Instead, you
// _must_ call DeserializeConstraintValue.
type ConstraintValue interface {
	isConstraintValue()
}

type ConstraintValueDeserializer struct {
	ConstraintValue
}

func DeserializeConstraintValue(b []byte) (*ConstraintValue, error) {
	var deserializer ConstraintValueDeserializer
	var result ConstraintValue
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.ConstraintValue
	return &result, nil
}

func (result *ConstraintValueDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing ConstraintValue as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}

	switch variantName {
	case "Term":
		var variant ConstraintValueTerm
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintValueDeserializer{variant}
		return nil
	case "Ref":
		var variant ConstraintValueRef
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintValueDeserializer{variant}
		return nil
	case "Field":
		var variant ConstraintValueField
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ConstraintValueDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize ConstraintValue: %s", string(b))
}

func (v ConstraintValueDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeConstraintValue(v.ConstraintValue)
}

func SerializeConstraintValue(variant ConstraintValue) ([]byte, error) {
	switch inner := variant.(type) {
	case ConstraintValueTerm:
		return json.Marshal(map[string]ConstraintValueTerm{
			"Term": inner,
		})
	case ConstraintValueRef:
		return json.Marshal(map[string]ConstraintValueRef{
			"Ref": inner,
		})
	case ConstraintValueField:
		return json.Marshal(map[string]ConstraintValueField{
			"Field": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

type DatumField Projection

func (variant DatumField) MarshalJSON() ([]byte, error) {
	return json.Marshal((Projection)(variant))
}

func (result *DatumField) UnmarshalJSON(b []byte) error {
	var inner Projection
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = DatumField(inner)
	return nil
}

func (DatumField) isDatum() {}

// DatumImmediate is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner Value
// as a field on a struct.
type DatumImmediate struct{ Value }

func (variant DatumImmediate) MarshalJSON() ([]byte, error) {
	return SerializeValue(variant.Value)
}

func (result *DatumImmediate) UnmarshalJSON(b []byte) error {
	v, err := DeserializeValue(b)
	if err != nil {
		return err
	}
	*result = DatumImmediate{Value: *v}
	return nil
}

func (DatumImmediate) isDatum() {}

// Datum enum
//
// The Rust enum type Datum is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Datum as a possibility for Datum.
//
// To make this clear, we prefix all variants with Datum
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Datum. Instead, you
// _must_ call DeserializeDatum.
type Datum interface {
	isDatum()
}

type DatumDeserializer struct {
	Datum
}

func DeserializeDatum(b []byte) (*Datum, error) {
	var deserializer DatumDeserializer
	var result Datum
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Datum
	return &result, nil
}

func (result *DatumDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Datum as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}

	switch variantName {
	case "Field":
		var variant DatumField
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = DatumDeserializer{variant}
		return nil
	case "Immediate":
		var variant DatumImmediate
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = DatumDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Datum: %s", string(b))
}

func (v DatumDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeDatum(v.Datum)
}

func SerializeDatum(variant Datum) ([]byte, error) {
	switch inner := variant.(type) {
	case DatumField:
		return json.Marshal(map[string]DatumField{
			"Field": inner,
		})
	case DatumImmediate:
		return json.Marshal(map[string]DatumImmediate{
			"Immediate": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

type DeclarationRole struct{}

func (DeclarationRole) isDeclaration() {}

type DeclarationPermission struct{}

func (DeclarationPermission) isDeclaration() {}

type DeclarationRelation Term

func (variant DeclarationRelation) MarshalJSON() ([]byte, error) {
	return json.Marshal((Term)(variant))
}

func (result *DeclarationRelation) UnmarshalJSON(b []byte) error {
	var inner Term
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = DeclarationRelation(inner)
	return nil
}

func (DeclarationRelation) isDeclaration() {}

// Declaration enum
//
// The Rust enum type Declaration is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Declaration as a possibility for Declaration.
//
// To make this clear, we prefix all variants with Declaration
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Declaration. Instead, you
// _must_ call DeserializeDeclaration.
type Declaration interface {
	isDeclaration()
}

type DeclarationDeserializer struct {
	Declaration
}

func DeserializeDeclaration(b []byte) (*Declaration, error) {
	var deserializer DeclarationDeserializer
	var result Declaration
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Declaration
	return &result, nil
}

func (result *DeclarationDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Declaration as an enum variant; expecting a single key")
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

		*result = DeclarationDeserializer{variant}
		return nil
	case "Permission":
		var variant DeclarationPermission
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = DeclarationDeserializer{variant}
		return nil
	case "Relation":
		var variant DeclarationRelation
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = DeclarationDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Declaration: %s", string(b))
}

func (v DeclarationDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeDeclaration(v.Declaration)
}

func SerializeDeclaration(variant Declaration) ([]byte, error) {
	switch inner := variant.(type) {
	case DeclarationRole:
		return json.Marshal("Role")
	case DeclarationPermission:
		return json.Marshal("Permission")
	case DeclarationRelation:
		return json.Marshal(map[string]DeclarationRelation{
			"Relation": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Dictionary struct
type Dictionary struct {
	// Fields
	Fields map[Symbol]Term `json:"fields"`
}

// ErrorKindParse is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner ParseError
// as a field on a struct.
type ErrorKindParse struct{ ParseError }

func (variant ErrorKindParse) MarshalJSON() ([]byte, error) {
	return SerializeParseError(variant.ParseError)
}

func (result *ErrorKindParse) UnmarshalJSON(b []byte) error {
	v, err := DeserializeParseError(b)
	if err != nil {
		return err
	}
	*result = ErrorKindParse{ParseError: *v}
	return nil
}

func (ErrorKindParse) isErrorKind() {}

// ErrorKindRuntime is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner RuntimeError
// as a field on a struct.
type ErrorKindRuntime struct{ RuntimeError }

func (variant ErrorKindRuntime) MarshalJSON() ([]byte, error) {
	return SerializeRuntimeError(variant.RuntimeError)
}

func (result *ErrorKindRuntime) UnmarshalJSON(b []byte) error {
	v, err := DeserializeRuntimeError(b)
	if err != nil {
		return err
	}
	*result = ErrorKindRuntime{RuntimeError: *v}
	return nil
}

func (ErrorKindRuntime) isErrorKind() {}

// ErrorKindOperational is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner OperationalError
// as a field on a struct.
type ErrorKindOperational struct{ OperationalError }

func (variant ErrorKindOperational) MarshalJSON() ([]byte, error) {
	return SerializeOperationalError(variant.OperationalError)
}

func (result *ErrorKindOperational) UnmarshalJSON(b []byte) error {
	v, err := DeserializeOperationalError(b)
	if err != nil {
		return err
	}
	*result = ErrorKindOperational{OperationalError: *v}
	return nil
}

func (ErrorKindOperational) isErrorKind() {}

// ErrorKindValidation is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner ValidationError
// as a field on a struct.
type ErrorKindValidation struct{ ValidationError }

func (variant ErrorKindValidation) MarshalJSON() ([]byte, error) {
	return SerializeValidationError(variant.ValidationError)
}

func (result *ErrorKindValidation) UnmarshalJSON(b []byte) error {
	v, err := DeserializeValidationError(b)
	if err != nil {
		return err
	}
	*result = ErrorKindValidation{ValidationError: *v}
	return nil
}

func (ErrorKindValidation) isErrorKind() {}

// ErrorKind enum
//
// The Rust enum type ErrorKind is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of ErrorKind as a possibility for ErrorKind.
//
// To make this clear, we prefix all variants with ErrorKind
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of ErrorKind. Instead, you
// _must_ call DeserializeErrorKind.
type ErrorKind interface {
	isErrorKind()
}

type ErrorKindDeserializer struct {
	ErrorKind
}

func DeserializeErrorKind(b []byte) (*ErrorKind, error) {
	var deserializer ErrorKindDeserializer
	var result ErrorKind
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.ErrorKind
	return &result, nil
}

func (result *ErrorKindDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing ErrorKind as an enum variant; expecting a single key")
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

		*result = ErrorKindDeserializer{variant}
		return nil
	case "Runtime":
		var variant ErrorKindRuntime
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ErrorKindDeserializer{variant}
		return nil
	case "Operational":
		var variant ErrorKindOperational
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ErrorKindDeserializer{variant}
		return nil
	case "Validation":
		var variant ErrorKindValidation
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ErrorKindDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize ErrorKind: %s", string(b))
}

func (v ErrorKindDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeErrorKind(v.ErrorKind)
}

func SerializeErrorKind(variant ErrorKind) ([]byte, error) {
	switch inner := variant.(type) {
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
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

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

// FetchRequest struct
type FetchRequest struct {
	// ClassTag
	ClassTag string `json:"class_tag"`
	// Constraints
	Constraints []Constraint `json:"constraints"`
}

// Filter struct
type Filter struct {
	// Root
	Root string `json:"root"`
	// Relations
	Relations []Relation `json:"relations"`
	// Conditions
	Conditions [][]Condition `json:"conditions"`
}

// FilterPlan struct
type FilterPlan struct {
	// ResultSets
	ResultSets []ResultSet `json:"result_sets"`
}

// FormattedPolarError struct
type FormattedPolarError struct {
	// Kind
	Kind ErrorKind `json:"kind"`
	// Formatted
	Formatted string `json:"formatted"`
}

func (result *FormattedPolarError) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawFormattedPolarError struct {
		// Kind
		Kind ErrorKindDeserializer `json:"kind"`
		// Formatted
		Formatted string `json:"formatted"`
	}
	var intermediate RawFormattedPolarError
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = FormattedPolarError{
		Kind:      intermediate.Kind.ErrorKind,
		Formatted: intermediate.Formatted,
	}
	return nil
}

func (v FormattedPolarError) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawFormattedPolarError struct {
		// Kind
		Kind ErrorKindDeserializer `json:"kind"`
		// Formatted
		Formatted string `json:"formatted"`
	}
	intermediate := RawFormattedPolarError{
		Kind:      ErrorKindDeserializer{ErrorKind: v.Kind},
		Formatted: v.Formatted,
	}
	return json.Marshal(intermediate)
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

func (result *Message) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawMessage struct {
		// Kind
		Kind MessageKindDeserializer `json:"kind"`
		// Msg
		Msg string `json:"msg"`
	}
	var intermediate RawMessage
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = Message{
		Kind: intermediate.Kind.MessageKind,
		Msg:  intermediate.Msg,
	}
	return nil
}

func (v Message) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawMessage struct {
		// Kind
		Kind MessageKindDeserializer `json:"kind"`
		// Msg
		Msg string `json:"msg"`
	}
	intermediate := RawMessage{
		Kind: MessageKindDeserializer{MessageKind: v.Kind},
		Msg:  v.Msg,
	}
	return json.Marshal(intermediate)
}

type MessageKindPrint struct{}

func (MessageKindPrint) isMessageKind() {}

type MessageKindWarning struct{}

func (MessageKindWarning) isMessageKind() {}

// MessageKind enum
//
// The Rust enum type MessageKind is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of MessageKind as a possibility for MessageKind.
//
// To make this clear, we prefix all variants with MessageKind
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of MessageKind. Instead, you
// _must_ call DeserializeMessageKind.
type MessageKind interface {
	isMessageKind()
}

type MessageKindDeserializer struct {
	MessageKind
}

func DeserializeMessageKind(b []byte) (*MessageKind, error) {
	var deserializer MessageKindDeserializer
	var result MessageKind
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.MessageKind
	return &result, nil
}

func (result *MessageKindDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing MessageKind as an enum variant; expecting a single key")
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

		*result = MessageKindDeserializer{variant}
		return nil
	case "Warning":
		var variant MessageKindWarning
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = MessageKindDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize MessageKind: %s", string(b))
}

func (v MessageKindDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeMessageKind(v.MessageKind)
}

func SerializeMessageKind(variant MessageKind) ([]byte, error) {
	switch inner := variant.(type) {
	case MessageKindPrint:
		return json.Marshal("Print")
	case MessageKindWarning:
		return json.Marshal("Warning")
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

type NodeRule Rule

func (variant NodeRule) MarshalJSON() ([]byte, error) {
	return json.Marshal((Rule)(variant))
}

func (result *NodeRule) UnmarshalJSON(b []byte) error {
	var inner Rule
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = NodeRule(inner)
	return nil
}

func (NodeRule) isNode() {}

type NodeTerm Term

func (variant NodeTerm) MarshalJSON() ([]byte, error) {
	return json.Marshal((Term)(variant))
}

func (result *NodeTerm) UnmarshalJSON(b []byte) error {
	var inner Term
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = NodeTerm(inner)
	return nil
}

func (NodeTerm) isNode() {}

// Node enum
//
// The Rust enum type Node is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Node as a possibility for Node.
//
// To make this clear, we prefix all variants with Node
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Node. Instead, you
// _must_ call DeserializeNode.
type Node interface {
	isNode()
}

type NodeDeserializer struct {
	Node
}

func DeserializeNode(b []byte) (*Node, error) {
	var deserializer NodeDeserializer
	var result Node
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Node
	return &result, nil
}

func (result *NodeDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Node as an enum variant; expecting a single key")
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

		*result = NodeDeserializer{variant}
		return nil
	case "Term":
		var variant NodeTerm
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = NodeDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Node: %s", string(b))
}

func (v NodeDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeNode(v.Node)
}

func SerializeNode(variant Node) ([]byte, error) {
	switch inner := variant.(type) {
	case NodeRule:
		return json.Marshal(map[string]NodeRule{
			"Rule": inner,
		})
	case NodeTerm:
		return json.Marshal(map[string]NodeTerm{
			"Term": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

type NumericInteger int64

func (variant NumericInteger) MarshalJSON() ([]byte, error) {
	return json.Marshal((int64)(variant))
}

func (result *NumericInteger) UnmarshalJSON(b []byte) error {
	var inner int64
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = NumericInteger(inner)
	return nil
}

func (NumericInteger) isNumeric() {}

type NumericFloat float64

func (variant NumericFloat) MarshalJSON() ([]byte, error) {
	return json.Marshal((float64)(variant))
}

func (result *NumericFloat) UnmarshalJSON(b []byte) error {
	var inner float64
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = NumericFloat(inner)
	return nil
}

func (NumericFloat) isNumeric() {}

// Numeric enum
//
// The Rust enum type Numeric is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Numeric as a possibility for Numeric.
//
// To make this clear, we prefix all variants with Numeric
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Numeric. Instead, you
// _must_ call DeserializeNumeric.
type Numeric interface {
	isNumeric()
}

type NumericDeserializer struct {
	Numeric
}

func DeserializeNumeric(b []byte) (*Numeric, error) {
	var deserializer NumericDeserializer
	var result Numeric
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Numeric
	return &result, nil
}

func (result *NumericDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Numeric as an enum variant; expecting a single key")
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

		*result = NumericDeserializer{variant}
		return nil
	case "Float":
		var variant NumericFloat
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = NumericDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Numeric: %s", string(b))
}

func (v NumericDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeNumeric(v.Numeric)
}

func SerializeNumeric(variant Numeric) ([]byte, error) {
	switch inner := variant.(type) {
	case NumericInteger:
		return json.Marshal(map[string]NumericInteger{
			"Integer": inner,
		})
	case NumericFloat:
		return json.Marshal(map[string]NumericFloat{
			"Float": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Operation struct
type Operation struct {
	// Operator
	Operator Operator `json:"operator"`
	// Args
	Args []Term `json:"args"`
}

func (result *Operation) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawOperation struct {
		// Operator
		Operator OperatorDeserializer `json:"operator"`
		// Args
		Args []Term `json:"args"`
	}
	var intermediate RawOperation
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = Operation{
		Operator: intermediate.Operator.Operator,
		Args:     intermediate.Args,
	}
	return nil
}

func (v Operation) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawOperation struct {
		// Operator
		Operator OperatorDeserializer `json:"operator"`
		// Args
		Args []Term `json:"args"`
	}
	intermediate := RawOperation{
		Operator: OperatorDeserializer{Operator: v.Operator},
		Args:     v.Args,
	}
	return json.Marshal(intermediate)
}

// OperationalErrorSerialization struct
type OperationalErrorSerialization struct {
	// Msg
	Msg string `json:"msg"`
}

func (OperationalErrorSerialization) isOperationalError() {}

type OperationalErrorUnknown struct{}

func (OperationalErrorUnknown) isOperationalError() {}

// OperationalError enum
//
// The Rust enum type OperationalError is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of OperationalError as a possibility for OperationalError.
//
// To make this clear, we prefix all variants with OperationalError
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of OperationalError. Instead, you
// _must_ call DeserializeOperationalError.
type OperationalError interface {
	isOperationalError()
}

type OperationalErrorDeserializer struct {
	OperationalError
}

func DeserializeOperationalError(b []byte) (*OperationalError, error) {
	var deserializer OperationalErrorDeserializer
	var result OperationalError
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.OperationalError
	return &result, nil
}

func (result *OperationalErrorDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing OperationalError as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}

	switch variantName {
	case "Serialization":
		var variant OperationalErrorSerialization
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperationalErrorDeserializer{variant}
		return nil
	case "Unknown":
		var variant OperationalErrorUnknown
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperationalErrorDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize OperationalError: %s", string(b))
}

func (v OperationalErrorDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeOperationalError(v.OperationalError)
}

func SerializeOperationalError(variant OperationalError) ([]byte, error) {
	switch inner := variant.(type) {
	case OperationalErrorSerialization:
		return json.Marshal(map[string]OperationalErrorSerialization{
			"Serialization": inner,
		})
	case OperationalErrorUnknown:
		return json.Marshal("Unknown")
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

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
//
// The Rust enum type Operator is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Operator as a possibility for Operator.
//
// To make this clear, we prefix all variants with Operator
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Operator. Instead, you
// _must_ call DeserializeOperator.
type Operator interface {
	isOperator()
}

type OperatorDeserializer struct {
	Operator
}

func DeserializeOperator(b []byte) (*Operator, error) {
	var deserializer OperatorDeserializer
	var result Operator
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Operator
	return &result, nil
}

func (result *OperatorDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Operator as an enum variant; expecting a single key")
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

		*result = OperatorDeserializer{variant}
		return nil
	case "Print":
		var variant OperatorPrint
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Cut":
		var variant OperatorCut
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "In":
		var variant OperatorIn
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Isa":
		var variant OperatorIsa
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "New":
		var variant OperatorNew
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Dot":
		var variant OperatorDot
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Not":
		var variant OperatorNot
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Mul":
		var variant OperatorMul
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Div":
		var variant OperatorDiv
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Mod":
		var variant OperatorMod
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Rem":
		var variant OperatorRem
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Add":
		var variant OperatorAdd
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Sub":
		var variant OperatorSub
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Eq":
		var variant OperatorEq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Geq":
		var variant OperatorGeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Leq":
		var variant OperatorLeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Neq":
		var variant OperatorNeq
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Gt":
		var variant OperatorGt
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Lt":
		var variant OperatorLt
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Unify":
		var variant OperatorUnify
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Or":
		var variant OperatorOr
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "And":
		var variant OperatorAnd
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "ForAll":
		var variant OperatorForAll
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	case "Assign":
		var variant OperatorAssign
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = OperatorDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Operator: %s", string(b))
}

func (v OperatorDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeOperator(v.Operator)
}

func SerializeOperator(variant Operator) ([]byte, error) {
	switch inner := variant.(type) {
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
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Parameter struct
type Parameter struct {
	// Parameter
	Parameter Term `json:"parameter"`
	// Specializer
	Specializer *Term `json:"specializer"`
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

// ParseError enum
//
// The Rust enum type ParseError is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of ParseError as a possibility for ParseError.
//
// To make this clear, we prefix all variants with ParseError
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of ParseError. Instead, you
// _must_ call DeserializeParseError.
type ParseError interface {
	isParseError()
}

type ParseErrorDeserializer struct {
	ParseError
}

func DeserializeParseError(b []byte) (*ParseError, error) {
	var deserializer ParseErrorDeserializer
	var result ParseError
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.ParseError
	return &result, nil
}

func (result *ParseErrorDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing ParseError as an enum variant; expecting a single key")
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

		*result = ParseErrorDeserializer{variant}
		return nil
	case "InvalidTokenCharacter":
		var variant ParseErrorInvalidTokenCharacter
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "InvalidToken":
		var variant ParseErrorInvalidToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "UnrecognizedEOF":
		var variant ParseErrorUnrecognizedEOF
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "UnrecognizedToken":
		var variant ParseErrorUnrecognizedToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "ExtraToken":
		var variant ParseErrorExtraToken
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "ReservedWord":
		var variant ParseErrorReservedWord
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "InvalidFloat":
		var variant ParseErrorInvalidFloat
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "WrongValueType":
		var variant ParseErrorWrongValueType
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	case "DuplicateKey":
		var variant ParseErrorDuplicateKey
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ParseErrorDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize ParseError: %s", string(b))
}

func (v ParseErrorDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeParseError(v.ParseError)
}

func SerializeParseError(variant ParseError) ([]byte, error) {
	switch inner := variant.(type) {
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
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

type PatternDictionary Dictionary

func (variant PatternDictionary) MarshalJSON() ([]byte, error) {
	return json.Marshal((Dictionary)(variant))
}

func (result *PatternDictionary) UnmarshalJSON(b []byte) error {
	var inner Dictionary
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = PatternDictionary(inner)
	return nil
}

func (PatternDictionary) isPattern() {}

type PatternInstance InstanceLiteral

func (variant PatternInstance) MarshalJSON() ([]byte, error) {
	return json.Marshal((InstanceLiteral)(variant))
}

func (result *PatternInstance) UnmarshalJSON(b []byte) error {
	var inner InstanceLiteral
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = PatternInstance(inner)
	return nil
}

func (PatternInstance) isPattern() {}

// Pattern enum
//
// The Rust enum type Pattern is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Pattern as a possibility for Pattern.
//
// To make this clear, we prefix all variants with Pattern
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Pattern. Instead, you
// _must_ call DeserializePattern.
type Pattern interface {
	isPattern()
}

type PatternDeserializer struct {
	Pattern
}

func DeserializePattern(b []byte) (*Pattern, error) {
	var deserializer PatternDeserializer
	var result Pattern
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Pattern
	return &result, nil
}

func (result *PatternDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Pattern as an enum variant; expecting a single key")
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

		*result = PatternDeserializer{variant}
		return nil
	case "Instance":
		var variant PatternInstance
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = PatternDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Pattern: %s", string(b))
}

func (v PatternDeserializer) MarshalJSON() ([]byte, error) {
	return SerializePattern(v.Pattern)
}

func SerializePattern(variant Pattern) ([]byte, error) {
	switch inner := variant.(type) {
	case PatternDictionary:
		return json.Marshal(map[string]PatternDictionary{
			"Dictionary": inner,
		})
	case PatternInstance:
		return json.Marshal(map[string]PatternInstance{
			"Instance": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Projection struct
//
// This mimics the Rust tuple type structure by constructing a
// field for each index in the tuple.
type Projection struct {
	V0 string
	V1 *string
}

func (result *Projection) UnmarshalJSON(b []byte) error {
	var jsonFields []json.RawMessage
	json.Unmarshal(b, &jsonFields)

	if len(jsonFields) != 2 {
		return fmt.Errorf("incorrect length for tuple. Expected %d, got %#v", 2, jsonFields)
	}

	var err error
	err = json.Unmarshal(jsonFields[0], &result.V0)
	if err != nil {
		return err
	}
	err = json.Unmarshal(jsonFields[1], &result.V1)
	if err != nil {
		return err
	}
	return nil
}

func (variant Projection) MarshalJSON() ([]byte, error) {
	fieldArray := []interface{}{
		variant.V0,
		variant.V1,
	}

	return json.Marshal(fieldArray)
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

func (result *QueryEventExternalOp) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawQueryEventExternalOp struct {
		// CallId
		CallId uint64 `json:"call_id"`
		// Operator
		Operator OperatorDeserializer `json:"operator"`
		// Args
		Args []Term `json:"args"`
	}
	var intermediate RawQueryEventExternalOp
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = QueryEventExternalOp{
		CallId:   intermediate.CallId,
		Operator: intermediate.Operator.Operator,
		Args:     intermediate.Args,
	}
	return nil
}

func (v QueryEventExternalOp) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawQueryEventExternalOp struct {
		// CallId
		CallId uint64 `json:"call_id"`
		// Operator
		Operator OperatorDeserializer `json:"operator"`
		// Args
		Args []Term `json:"args"`
	}
	intermediate := RawQueryEventExternalOp{
		CallId:   v.CallId,
		Operator: OperatorDeserializer{Operator: v.Operator},
		Args:     v.Args,
	}
	return json.Marshal(intermediate)
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
//
// The Rust enum type QueryEvent is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of QueryEvent as a possibility for QueryEvent.
//
// To make this clear, we prefix all variants with QueryEvent
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of QueryEvent. Instead, you
// _must_ call DeserializeQueryEvent.
type QueryEvent interface {
	isQueryEvent()
}

type QueryEventDeserializer struct {
	QueryEvent
}

func DeserializeQueryEvent(b []byte) (*QueryEvent, error) {
	var deserializer QueryEventDeserializer
	var result QueryEvent
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.QueryEvent
	return &result, nil
}

func (result *QueryEventDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing QueryEvent as an enum variant; expecting a single key")
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

		*result = QueryEventDeserializer{variant}
		return nil
	case "Done":
		var variant QueryEventDone
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "Debug":
		var variant QueryEventDebug
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "MakeExternal":
		var variant QueryEventMakeExternal
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "ExternalCall":
		var variant QueryEventExternalCall
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "ExternalIsa":
		var variant QueryEventExternalIsa
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "ExternalIsaWithPath":
		var variant QueryEventExternalIsaWithPath
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "ExternalIsSubSpecializer":
		var variant QueryEventExternalIsSubSpecializer
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "ExternalIsSubclass":
		var variant QueryEventExternalIsSubclass
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "Result":
		var variant QueryEventResult
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "ExternalOp":
		var variant QueryEventExternalOp
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	case "NextExternal":
		var variant QueryEventNextExternal
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = QueryEventDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize QueryEvent: %s", string(b))
}

func (v QueryEventDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeQueryEvent(v.QueryEvent)
}

func SerializeQueryEvent(variant QueryEvent) ([]byte, error) {
	switch inner := variant.(type) {
	case QueryEventNone:
		return json.Marshal("None")
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
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Ref struct
type Ref struct {
	// Field
	Field *string `json:"field"`
	// ResultId
	ResultId uint64 `json:"result_id"`
}

// Relation struct
//
// This mimics the Rust tuple type structure by constructing a
// field for each index in the tuple.
type Relation struct {
	V0 string
	V1 string
	V2 string
}

func (result *Relation) UnmarshalJSON(b []byte) error {
	var jsonFields []json.RawMessage
	json.Unmarshal(b, &jsonFields)

	if len(jsonFields) != 3 {
		return fmt.Errorf("incorrect length for tuple. Expected %d, got %#v", 3, jsonFields)
	}

	var err error
	err = json.Unmarshal(jsonFields[0], &result.V0)
	if err != nil {
		return err
	}
	err = json.Unmarshal(jsonFields[1], &result.V1)
	if err != nil {
		return err
	}
	err = json.Unmarshal(jsonFields[2], &result.V2)
	if err != nil {
		return err
	}
	return nil
}

func (variant Relation) MarshalJSON() ([]byte, error) {
	fieldArray := []interface{}{
		variant.V0,
		variant.V1,
		variant.V2,
	}

	return json.Marshal(fieldArray)
}

// ResultSet struct
type ResultSet struct {
	// Requests
	Requests map[uint64]FetchRequest `json:"requests"`
	// ResolveOrder
	ResolveOrder []uint64 `json:"resolve_order"`
	// ResultId
	ResultId uint64 `json:"result_id"`
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
	// Msg
	Msg string `json:"msg"`
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

// RuntimeErrorInvalidState struct
type RuntimeErrorInvalidState struct {
	// Msg
	Msg string `json:"msg"`
}

func (RuntimeErrorInvalidState) isRuntimeError() {}

type RuntimeErrorMultipleLoadError struct{}

func (RuntimeErrorMultipleLoadError) isRuntimeError() {}

// RuntimeError enum
//
// The Rust enum type RuntimeError is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of RuntimeError as a possibility for RuntimeError.
//
// To make this clear, we prefix all variants with RuntimeError
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of RuntimeError. Instead, you
// _must_ call DeserializeRuntimeError.
type RuntimeError interface {
	isRuntimeError()
}

type RuntimeErrorDeserializer struct {
	RuntimeError
}

func DeserializeRuntimeError(b []byte) (*RuntimeError, error) {
	var deserializer RuntimeErrorDeserializer
	var result RuntimeError
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.RuntimeError
	return &result, nil
}

func (result *RuntimeErrorDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing RuntimeError as an enum variant; expecting a single key")
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

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "Unsupported":
		var variant RuntimeErrorUnsupported
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "TypeError":
		var variant RuntimeErrorTypeError
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "StackOverflow":
		var variant RuntimeErrorStackOverflow
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "QueryTimeout":
		var variant RuntimeErrorQueryTimeout
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "Application":
		var variant RuntimeErrorApplication
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "IncompatibleBindings":
		var variant RuntimeErrorIncompatibleBindings
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "UnhandledPartial":
		var variant RuntimeErrorUnhandledPartial
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "DataFilteringFieldMissing":
		var variant RuntimeErrorDataFilteringFieldMissing
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "DataFilteringUnsupportedOp":
		var variant RuntimeErrorDataFilteringUnsupportedOp
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "InvalidRegistration":
		var variant RuntimeErrorInvalidRegistration
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "InvalidState":
		var variant RuntimeErrorInvalidState
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	case "MultipleLoadError":
		var variant RuntimeErrorMultipleLoadError
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = RuntimeErrorDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize RuntimeError: %s", string(b))
}

func (v RuntimeErrorDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeRuntimeError(v.RuntimeError)
}

func SerializeRuntimeError(variant RuntimeError) ([]byte, error) {
	switch inner := variant.(type) {
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
	case RuntimeErrorInvalidState:
		return json.Marshal(map[string]RuntimeErrorInvalidState{
			"InvalidState": inner,
		})
	case RuntimeErrorMultipleLoadError:
		return json.Marshal("MultipleLoadError")
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// Source struct
type Source struct {
	// Filename
	Filename *string `json:"filename"`
	// Src
	Src string `json:"src"`
}

type Symbol string

func (variant Symbol) MarshalJSON() ([]byte, error) {
	return json.Marshal((string)(variant))
}

func (result *Symbol) UnmarshalJSON(b []byte) error {
	var inner string
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = Symbol(inner)
	return nil
}

// Term struct
type Term struct {
	// Value
	Value Value `json:"value"`
}

func (result *Term) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawTerm struct {
		// Value
		Value ValueDeserializer `json:"value"`
	}
	var intermediate RawTerm
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = Term{
		Value: intermediate.Value.Value,
	}
	return nil
}

func (v Term) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawTerm struct {
		// Value
		Value ValueDeserializer `json:"value"`
	}
	intermediate := RawTerm{
		Value: ValueDeserializer{Value: v.Value},
	}
	return json.Marshal(intermediate)
}

// Trace struct
type Trace struct {
	// Node
	Node Node `json:"node"`
	// Children
	Children []Trace `json:"children"`
}

func (result *Trace) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawTrace struct {
		// Node
		Node NodeDeserializer `json:"node"`
		// Children
		Children []Trace `json:"children"`
	}
	var intermediate RawTrace
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = Trace{
		Node:     intermediate.Node.Node,
		Children: intermediate.Children,
	}
	return nil
}

func (v Trace) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawTrace struct {
		// Node
		Node NodeDeserializer `json:"node"`
		// Children
		Children []Trace `json:"children"`
	}
	intermediate := RawTrace{
		Node:     NodeDeserializer{Node: v.Node},
		Children: v.Children,
	}
	return json.Marshal(intermediate)
}

// TraceResult struct
type TraceResult struct {
	// Trace
	Trace Trace `json:"trace"`
	// Formatted
	Formatted string `json:"formatted"`
}

// TypeBase struct
type TypeBase struct {
	// ClassTag
	ClassTag string `json:"class_tag"`
}

func (TypeBase) isType() {}

// TypeRelation struct
type TypeRelation struct {
	// Kind
	Kind string `json:"kind"`
	// OtherClassTag
	OtherClassTag string `json:"other_class_tag"`
	// MyField
	MyField string `json:"my_field"`
	// OtherField
	OtherField string `json:"other_field"`
}

func (TypeRelation) isType() {}

// Type enum
//
// The Rust enum type Type is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Type as a possibility for Type.
//
// To make this clear, we prefix all variants with Type
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Type. Instead, you
// _must_ call DeserializeType.
type Type interface {
	isType()
}

type TypeDeserializer struct {
	Type
}

func DeserializeType(b []byte) (*Type, error) {
	var deserializer TypeDeserializer
	var result Type
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Type
	return &result, nil
}

func (result *TypeDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Type as an enum variant; expecting a single key")
		}
		for k, v := range rawMap {
			variantName = k
			variantValue = &v
		}
	}

	switch variantName {
	case "Base":
		var variant TypeBase
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = TypeDeserializer{variant}
		return nil
	case "Relation":
		var variant TypeRelation
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = TypeDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Type: %s", string(b))
}

func (v TypeDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeType(v.Type)
}

func SerializeType(variant Type) ([]byte, error) {
	switch inner := variant.(type) {
	case TypeBase:
		return json.Marshal(map[string]TypeBase{
			"Base": inner,
		})
	case TypeRelation:
		return json.Marshal(map[string]TypeRelation{
			"Relation": inner,
		})
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// ValidationErrorFileLoading struct
type ValidationErrorFileLoading struct {
	// Source
	Source Source `json:"source"`
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

func (result *ValidationErrorDuplicateResourceBlockDeclaration) UnmarshalJSON(b []byte) error {
	// This struct contains enums which need to be deserialized using the intermediate
	// <TypeName>Deserializer structs
	type RawValidationErrorDuplicateResourceBlockDeclaration struct {
		// Resource
		Resource Term `json:"resource"`
		// Declaration
		Declaration Term `json:"declaration"`
		// Existing
		Existing DeclarationDeserializer `json:"existing"`
		// New
		New DeclarationDeserializer `json:"new"`
	}
	var intermediate RawValidationErrorDuplicateResourceBlockDeclaration
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}

	*result = ValidationErrorDuplicateResourceBlockDeclaration{
		Resource:    intermediate.Resource,
		Declaration: intermediate.Declaration,
		Existing:    intermediate.Existing.Declaration,
		New:         intermediate.New.Declaration,
	}
	return nil
}

func (v ValidationErrorDuplicateResourceBlockDeclaration) MarshalJSON() ([]byte, error) {

	// This struct contains enums which need to be serialized using the intermediate
	// <TypeName>Deserializer structs
	type RawValidationErrorDuplicateResourceBlockDeclaration struct {
		// Resource
		Resource Term `json:"resource"`
		// Declaration
		Declaration Term `json:"declaration"`
		// Existing
		Existing DeclarationDeserializer `json:"existing"`
		// New
		New DeclarationDeserializer `json:"new"`
	}
	intermediate := RawValidationErrorDuplicateResourceBlockDeclaration{
		Resource:    v.Resource,
		Declaration: v.Declaration,
		Existing:    DeclarationDeserializer{Declaration: v.Existing},
		New:         DeclarationDeserializer{Declaration: v.New},
	}
	return json.Marshal(intermediate)
}

func (ValidationErrorDuplicateResourceBlockDeclaration) isValidationError() {}

// ValidationError enum
//
// The Rust enum type ValidationError is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of ValidationError as a possibility for ValidationError.
//
// To make this clear, we prefix all variants with ValidationError
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of ValidationError. Instead, you
// _must_ call DeserializeValidationError.
type ValidationError interface {
	isValidationError()
}

type ValidationErrorDeserializer struct {
	ValidationError
}

func DeserializeValidationError(b []byte) (*ValidationError, error) {
	var deserializer ValidationErrorDeserializer
	var result ValidationError
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.ValidationError
	return &result, nil
}

func (result *ValidationErrorDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing ValidationError as an enum variant; expecting a single key")
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

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "MissingRequiredRule":
		var variant ValidationErrorMissingRequiredRule
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "InvalidRule":
		var variant ValidationErrorInvalidRule
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "InvalidRuleType":
		var variant ValidationErrorInvalidRuleType
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "UndefinedRuleCall":
		var variant ValidationErrorUndefinedRuleCall
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "ResourceBlock":
		var variant ValidationErrorResourceBlock
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "SingletonVariable":
		var variant ValidationErrorSingletonVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "UnregisteredClass":
		var variant ValidationErrorUnregisteredClass
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	case "DuplicateResourceBlockDeclaration":
		var variant ValidationErrorDuplicateResourceBlockDeclaration
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValidationErrorDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize ValidationError: %s", string(b))
}

func (v ValidationErrorDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeValidationError(v.ValidationError)
}

func SerializeValidationError(variant ValidationError) ([]byte, error) {
	switch inner := variant.(type) {
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
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}

// ValueNumber is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner Numeric
// as a field on a struct.
type ValueNumber struct{ Numeric }

func (variant ValueNumber) MarshalJSON() ([]byte, error) {
	return SerializeNumeric(variant.Numeric)
}

func (result *ValueNumber) UnmarshalJSON(b []byte) error {
	v, err := DeserializeNumeric(b)
	if err != nil {
		return err
	}
	*result = ValueNumber{Numeric: *v}
	return nil
}

func (ValueNumber) isValue() {}

type ValueString string

func (variant ValueString) MarshalJSON() ([]byte, error) {
	return json.Marshal((string)(variant))
}

func (result *ValueString) UnmarshalJSON(b []byte) error {
	var inner string
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueString(inner)
	return nil
}

func (ValueString) isValue() {}

type ValueBoolean bool

func (variant ValueBoolean) MarshalJSON() ([]byte, error) {
	return json.Marshal((bool)(variant))
}

func (result *ValueBoolean) UnmarshalJSON(b []byte) error {
	var inner bool
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueBoolean(inner)
	return nil
}

func (ValueBoolean) isValue() {}

type ValueExternalInstance ExternalInstance

func (variant ValueExternalInstance) MarshalJSON() ([]byte, error) {
	return json.Marshal((ExternalInstance)(variant))
}

func (result *ValueExternalInstance) UnmarshalJSON(b []byte) error {
	var inner ExternalInstance
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueExternalInstance(inner)
	return nil
}

func (ValueExternalInstance) isValue() {}

type ValueDictionary Dictionary

func (variant ValueDictionary) MarshalJSON() ([]byte, error) {
	return json.Marshal((Dictionary)(variant))
}

func (result *ValueDictionary) UnmarshalJSON(b []byte) error {
	var inner Dictionary
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueDictionary(inner)
	return nil
}

func (ValueDictionary) isValue() {}

// ValuePattern is a newtype struct wrapping a Rust enum
// Since we convert enums to Go interfaces, it's
// a little easier for us to wrap the inner Pattern
// as a field on a struct.
type ValuePattern struct{ Pattern }

func (variant ValuePattern) MarshalJSON() ([]byte, error) {
	return SerializePattern(variant.Pattern)
}

func (result *ValuePattern) UnmarshalJSON(b []byte) error {
	v, err := DeserializePattern(b)
	if err != nil {
		return err
	}
	*result = ValuePattern{Pattern: *v}
	return nil
}

func (ValuePattern) isValue() {}

type ValueCall Call

func (variant ValueCall) MarshalJSON() ([]byte, error) {
	return json.Marshal((Call)(variant))
}

func (result *ValueCall) UnmarshalJSON(b []byte) error {
	var inner Call
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueCall(inner)
	return nil
}

func (ValueCall) isValue() {}

type ValueList []Term

func (variant ValueList) MarshalJSON() ([]byte, error) {
	return json.Marshal(([]Term)(variant))
}

func (result *ValueList) UnmarshalJSON(b []byte) error {
	var inner []Term
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueList(inner)
	return nil
}

func (ValueList) isValue() {}

type ValueVariable Symbol

func (variant ValueVariable) MarshalJSON() ([]byte, error) {
	return json.Marshal((Symbol)(variant))
}

func (result *ValueVariable) UnmarshalJSON(b []byte) error {
	var inner Symbol
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueVariable(inner)
	return nil
}

func (ValueVariable) isValue() {}

type ValueRestVariable Symbol

func (variant ValueRestVariable) MarshalJSON() ([]byte, error) {
	return json.Marshal((Symbol)(variant))
}

func (result *ValueRestVariable) UnmarshalJSON(b []byte) error {
	var inner Symbol
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueRestVariable(inner)
	return nil
}

func (ValueRestVariable) isValue() {}

type ValueExpression Operation

func (variant ValueExpression) MarshalJSON() ([]byte, error) {
	return json.Marshal((Operation)(variant))
}

func (result *ValueExpression) UnmarshalJSON(b []byte) error {
	var inner Operation
	err := json.Unmarshal(b, &inner)
	if err != nil {
		return err
	}
	*result = ValueExpression(inner)
	return nil
}

func (ValueExpression) isValue() {}

// Value enum
//
// The Rust enum type Value is represented in Go with an interfact
// this allows us to mimic the sum type by accepting any variant
// of Value as a possibility for Value.
//
// To make this clear, we prefix all variants with Value
//
// The downside of this approach is that you cannot directly
// serialize or deserialize instances of Value. Instead, you
// _must_ call DeserializeValue.
type Value interface {
	isValue()
}

type ValueDeserializer struct {
	Value
}

func DeserializeValue(b []byte) (*Value, error) {
	var deserializer ValueDeserializer
	var result Value
	err := json.Unmarshal(b, &deserializer)
	if err != nil {
		return nil, err
	}
	result = deserializer.Value
	return &result, nil
}

func (result *ValueDeserializer) UnmarshalJSON(b []byte) error {
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
			return errors.New("deserializing Value as an enum variant; expecting a single key")
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

		*result = ValueDeserializer{variant}
		return nil
	case "String":
		var variant ValueString
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "Boolean":
		var variant ValueBoolean
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "ExternalInstance":
		var variant ValueExternalInstance
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "Dictionary":
		var variant ValueDictionary
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "Pattern":
		var variant ValuePattern
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "Call":
		var variant ValueCall
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "List":
		var variant ValueList
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "Variable":
		var variant ValueVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "RestVariable":
		var variant ValueRestVariable
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	case "Expression":
		var variant ValueExpression
		if variantValue != nil {
			err := json.Unmarshal(*variantValue, &variant)
			if err != nil {
				return err
			}
		}

		*result = ValueDeserializer{variant}
		return nil
	}

	return fmt.Errorf("cannot deserialize Value: %s", string(b))
}

func (v ValueDeserializer) MarshalJSON() ([]byte, error) {
	return SerializeValue(v.Value)
}

func SerializeValue(variant Value) ([]byte, error) {
	switch inner := variant.(type) {
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
	default:
		return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
	}

}
