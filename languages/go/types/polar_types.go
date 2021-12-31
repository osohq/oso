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

type ComparisonEq struct {}

func (ComparisonEq) isComparison() {}
type ComparisonNeq struct {}

func (ComparisonNeq) isComparison() {}
type ComparisonIn struct {}

func (ComparisonIn) isComparison() {}
type ComparisonNin struct {}

func (ComparisonNin) isComparison() {}
type ComparisonLt struct {}

func (ComparisonLt) isComparison() {}
type ComparisonLeq struct {}

func (ComparisonLeq) isComparison() {}
type ComparisonGt struct {}

func (ComparisonGt) isComparison() {}
type ComparisonGeq struct {}

func (ComparisonGeq) isComparison() {}
// Comparison enum
type ComparisonVariant interface {
    isComparison()
}

type Comparison struct {
    ComparisonVariant
}

type ComparisonDeserializer struct {
   Inner Comparison
}

func (result *Comparison) UnmarshalJSON(b []byte) error {
    var deserializer ComparisonDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "Neq":
        var variant ComparisonNeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "In":
        var variant ComparisonIn
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "Nin":
        var variant ComparisonNin
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "Lt":
        var variant ComparisonLt
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "Leq":
        var variant ComparisonLeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "Gt":
        var variant ComparisonGt
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    case "Geq":
        var variant ComparisonGeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ComparisonDeserializer { Inner: Comparison { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Comparison: %s", string(b))
}


func (variant Comparison) MarshalJSON() ([]byte, error) {
    switch inner := variant.ComparisonVariant.(type) {
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
type Condition struct {
    // 0
    V0 Datum
    // 1
    V1 Comparison
    // 2
    V2 Datum
}

func (result *Condition) UnmarshalJSON(b []byte) error {
    var jsonFields []json.RawMessage
    json.Unmarshal(b, &jsonFields)

    if (len(jsonFields) != 3) {
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

    *result = Constraint {
        Kind: intermediate.Kind.Inner,
        Field: intermediate.Field,  
        Value: intermediate.Value.Inner,
    }
    return nil
}
type ConstraintKindEq struct {}

func (ConstraintKindEq) isConstraintKind() {}
type ConstraintKindIn struct {}

func (ConstraintKindIn) isConstraintKind() {}
type ConstraintKindContains struct {}

func (ConstraintKindContains) isConstraintKind() {}
type ConstraintKindNeq struct {}

func (ConstraintKindNeq) isConstraintKind() {}
type ConstraintKindNin struct {}

func (ConstraintKindNin) isConstraintKind() {}
// ConstraintKind enum
type ConstraintKindVariant interface {
    isConstraintKind()
}

type ConstraintKind struct {
    ConstraintKindVariant
}

type ConstraintKindDeserializer struct {
   Inner ConstraintKind
}

func (result *ConstraintKind) UnmarshalJSON(b []byte) error {
    var deserializer ConstraintKindDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintKindDeserializer { Inner: ConstraintKind { variant } }
        return nil
    case "In":
        var variant ConstraintKindIn
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintKindDeserializer { Inner: ConstraintKind { variant } }
        return nil
    case "Contains":
        var variant ConstraintKindContains
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintKindDeserializer { Inner: ConstraintKind { variant } }
        return nil
    case "Neq":
        var variant ConstraintKindNeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintKindDeserializer { Inner: ConstraintKind { variant } }
        return nil
    case "Nin":
        var variant ConstraintKindNin
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintKindDeserializer { Inner: ConstraintKind { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize ConstraintKind: %s", string(b))
}


func (variant ConstraintKind) MarshalJSON() ([]byte, error) {
    switch inner := variant.ConstraintKindVariant.(type) {
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
// ConstraintValueTerm newtype
type ConstraintValueTerm Term

func (variant ConstraintValueTerm) MarshalJSON() ([]byte, error) {
    return json.Marshal(Term(variant))
}

func (variant *ConstraintValueTerm) UnmarshalJSON(b []byte) (error) {
    inner := Term(*variant)
    err := json.Unmarshal(b, &inner)
    *variant = ConstraintValueTerm(inner)
    return err
}

func (ConstraintValueTerm) isConstraintValue() {}
// ConstraintValueRef newtype
type ConstraintValueRef Ref

func (variant ConstraintValueRef) MarshalJSON() ([]byte, error) {
    return json.Marshal(Ref(variant))
}

func (variant *ConstraintValueRef) UnmarshalJSON(b []byte) (error) {
    inner := Ref(*variant)
    err := json.Unmarshal(b, &inner)
    *variant = ConstraintValueRef(inner)
    return err
}

func (ConstraintValueRef) isConstraintValue() {}
// ConstraintValueField newtype
type ConstraintValueField string

func (variant ConstraintValueField) MarshalJSON() ([]byte, error) {
    return json.Marshal(string(variant))
}

func (variant *ConstraintValueField) UnmarshalJSON(b []byte) (error) {
    inner := string(*variant)
    err := json.Unmarshal(b, &inner)
    *variant = ConstraintValueField(inner)
    return err
}

func (ConstraintValueField) isConstraintValue() {}
// ConstraintValue enum
type ConstraintValueVariant interface {
    isConstraintValue()
}

type ConstraintValue struct {
    ConstraintValueVariant
}

type ConstraintValueDeserializer struct {
   Inner ConstraintValue
}

func (result *ConstraintValue) UnmarshalJSON(b []byte) error {
    var deserializer ConstraintValueDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintValueDeserializer { Inner: ConstraintValue { variant } }
        return nil
    case "Ref":
        var variant ConstraintValueRef
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintValueDeserializer { Inner: ConstraintValue { variant } }
        return nil
    case "Field":
        var variant ConstraintValueField
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ConstraintValueDeserializer { Inner: ConstraintValue { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize ConstraintValue: %s", string(b))
}


func (variant ConstraintValue) MarshalJSON() ([]byte, error) {
    switch inner := variant.ConstraintValueVariant.(type) {
    case ConstraintValueTerm:
        return json.Marshal(map[string]ConstraintValueTerm {
            "Term": inner,
        });
    case ConstraintValueRef:
        return json.Marshal(map[string]ConstraintValueRef {
            "Ref": inner,
        });
    case ConstraintValueField:
        return json.Marshal(map[string]ConstraintValueField {
            "Field": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
// DatumField newtype
type DatumField Projection

func (variant DatumField) MarshalJSON() ([]byte, error) {
    return json.Marshal(Projection(variant))
}

func (variant *DatumField) UnmarshalJSON(b []byte) (error) {
    inner := Projection(*variant)
    err := json.Unmarshal(b, &inner)
    *variant = DatumField(inner)
    return err
}

func (DatumField) isDatum() {}
// DatumImmediate newtype
type DatumImmediate Value

func (variant DatumImmediate) MarshalJSON() ([]byte, error) {
    return json.Marshal(Value(variant))
}

func (variant *DatumImmediate) UnmarshalJSON(b []byte) (error) {
    inner := Value(*variant)
    err := json.Unmarshal(b, &inner)
    *variant = DatumImmediate(inner)
    return err
}

func (DatumImmediate) isDatum() {}
// Datum enum
type DatumVariant interface {
    isDatum()
}

type Datum struct {
    DatumVariant
}

type DatumDeserializer struct {
   Inner Datum
}

func (result *Datum) UnmarshalJSON(b []byte) error {
    var deserializer DatumDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = DatumDeserializer { Inner: Datum { variant } }
        return nil
    case "Immediate":
        var variant DatumImmediate
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = DatumDeserializer { Inner: Datum { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Datum: %s", string(b))
}


func (variant Datum) MarshalJSON() ([]byte, error) {
    switch inner := variant.DatumVariant.(type) {
    case DatumField:
        return json.Marshal(map[string]DatumField {
            "Field": inner,
        });
    case DatumImmediate:
        return json.Marshal(map[string]DatumImmediate {
            "Immediate": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
type DeclarationRole struct {}

func (DeclarationRole) isDeclaration() {}
type DeclarationPermission struct {}

func (DeclarationPermission) isDeclaration() {}
// DeclarationRelation newtype
type DeclarationRelation Term

func (variant DeclarationRelation) MarshalJSON() ([]byte, error) {
    return json.Marshal(Term(variant))
}

func (variant *DeclarationRelation) UnmarshalJSON(b []byte) (error) {
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

type DeclarationDeserializer struct {
   Inner Declaration
}

func (result *Declaration) UnmarshalJSON(b []byte) error {
    var deserializer DeclarationDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = DeclarationDeserializer { Inner: Declaration { variant } }
        return nil
    case "Permission":
        var variant DeclarationPermission
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = DeclarationDeserializer { Inner: Declaration { variant } }
        return nil
    case "Relation":
        var variant DeclarationRelation
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = DeclarationDeserializer { Inner: Declaration { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Declaration: %s", string(b))
}


func (variant Declaration) MarshalJSON() ([]byte, error) {
    switch inner := variant.DeclarationVariant.(type) {
    case DeclarationRole:
        return json.Marshal("Role")
    case DeclarationPermission:
        return json.Marshal("Permission")
    case DeclarationRelation:
        return json.Marshal(map[string]DeclarationRelation {
            "Relation": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

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

func (variant *ErrorKindParse) UnmarshalJSON(b []byte) (error) {
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

func (variant *ErrorKindRuntime) UnmarshalJSON(b []byte) (error) {
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

func (variant *ErrorKindOperational) UnmarshalJSON(b []byte) (error) {
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

func (variant *ErrorKindValidation) UnmarshalJSON(b []byte) (error) {
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

type ErrorKindDeserializer struct {
   Inner ErrorKind
}

func (result *ErrorKind) UnmarshalJSON(b []byte) error {
    var deserializer ErrorKindDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ErrorKindDeserializer { Inner: ErrorKind { variant } }
        return nil
    case "Runtime":
        var variant ErrorKindRuntime
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ErrorKindDeserializer { Inner: ErrorKind { variant } }
        return nil
    case "Operational":
        var variant ErrorKindOperational
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ErrorKindDeserializer { Inner: ErrorKind { variant } }
        return nil
    case "Validation":
        var variant ErrorKindValidation
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ErrorKindDeserializer { Inner: ErrorKind { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize ErrorKind: %s", string(b))
}


func (variant ErrorKind) MarshalJSON() ([]byte, error) {
    switch inner := variant.ErrorKindVariant.(type) {
    case ErrorKindParse:
        return json.Marshal(map[string]ErrorKindParse {
            "Parse": inner,
        });
    case ErrorKindRuntime:
        return json.Marshal(map[string]ErrorKindRuntime {
            "Runtime": inner,
        });
    case ErrorKindOperational:
        return json.Marshal(map[string]ErrorKindOperational {
            "Operational": inner,
        });
    case ErrorKindValidation:
        return json.Marshal(map[string]ErrorKindValidation {
            "Validation": inner,
        });
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

    *result = FormattedPolarError {
        Kind: intermediate.Kind.Inner,
        Formatted: intermediate.Formatted,  
    }
    return nil
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

    *result = Message {
        Kind: intermediate.Kind.Inner,
        Msg: intermediate.Msg,  
    }
    return nil
}
type MessageKindPrint struct {}

func (MessageKindPrint) isMessageKind() {}
type MessageKindWarning struct {}

func (MessageKindWarning) isMessageKind() {}
// MessageKind enum
type MessageKindVariant interface {
    isMessageKind()
}

type MessageKind struct {
    MessageKindVariant
}

type MessageKindDeserializer struct {
   Inner MessageKind
}

func (result *MessageKind) UnmarshalJSON(b []byte) error {
    var deserializer MessageKindDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = MessageKindDeserializer { Inner: MessageKind { variant } }
        return nil
    case "Warning":
        var variant MessageKindWarning
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = MessageKindDeserializer { Inner: MessageKind { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize MessageKind: %s", string(b))
}


func (variant MessageKind) MarshalJSON() ([]byte, error) {
    switch inner := variant.MessageKindVariant.(type) {
    case MessageKindPrint:
        return json.Marshal("Print")
    case MessageKindWarning:
        return json.Marshal("Warning")
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
// NodeRule newtype
type NodeRule Rule

func (variant NodeRule) MarshalJSON() ([]byte, error) {
    return json.Marshal(Rule(variant))
}

func (variant *NodeRule) UnmarshalJSON(b []byte) (error) {
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

func (variant *NodeTerm) UnmarshalJSON(b []byte) (error) {
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

type NodeDeserializer struct {
   Inner Node
}

func (result *Node) UnmarshalJSON(b []byte) error {
    var deserializer NodeDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = NodeDeserializer { Inner: Node { variant } }
        return nil
    case "Term":
        var variant NodeTerm
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = NodeDeserializer { Inner: Node { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Node: %s", string(b))
}


func (variant Node) MarshalJSON() ([]byte, error) {
    switch inner := variant.NodeVariant.(type) {
    case NodeRule:
        return json.Marshal(map[string]NodeRule {
            "Rule": inner,
        });
    case NodeTerm:
        return json.Marshal(map[string]NodeTerm {
            "Term": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
// NumericInteger newtype
type NumericInteger int64

func (variant NumericInteger) MarshalJSON() ([]byte, error) {
    return json.Marshal(int64(variant))
}

func (variant *NumericInteger) UnmarshalJSON(b []byte) (error) {
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

func (variant *NumericFloat) UnmarshalJSON(b []byte) (error) {
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

type NumericDeserializer struct {
   Inner Numeric
}

func (result *Numeric) UnmarshalJSON(b []byte) error {
    var deserializer NumericDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = NumericDeserializer { Inner: Numeric { variant } }
        return nil
    case "Float":
        var variant NumericFloat
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = NumericDeserializer { Inner: Numeric { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Numeric: %s", string(b))
}


func (variant Numeric) MarshalJSON() ([]byte, error) {
    switch inner := variant.NumericVariant.(type) {
    case NumericInteger:
        return json.Marshal(map[string]NumericInteger {
            "Integer": inner,
        });
    case NumericFloat:
        return json.Marshal(map[string]NumericFloat {
            "Float": inner,
        });
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

    *result = Operation {
        Operator: intermediate.Operator.Inner,
        Args: intermediate.Args,  
    }
    return nil
}
// OperationalErrorSerialization struct
type OperationalErrorSerialization struct {
    // Msg
    Msg string `json:"msg"`
}


func (OperationalErrorSerialization) isOperationalError() {}
type OperationalErrorUnknown struct {}

func (OperationalErrorUnknown) isOperationalError() {}
// OperationalError enum
type OperationalErrorVariant interface {
    isOperationalError()
}

type OperationalError struct {
    OperationalErrorVariant
}

type OperationalErrorDeserializer struct {
   Inner OperationalError
}

func (result *OperationalError) UnmarshalJSON(b []byte) error {
    var deserializer OperationalErrorDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperationalErrorDeserializer { Inner: OperationalError { variant } }
        return nil
    case "Unknown":
        var variant OperationalErrorUnknown
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperationalErrorDeserializer { Inner: OperationalError { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize OperationalError: %s", string(b))
}


func (variant OperationalError) MarshalJSON() ([]byte, error) {
    switch inner := variant.OperationalErrorVariant.(type) {
    case OperationalErrorSerialization:
        return json.Marshal(map[string]OperationalErrorSerialization {
            "Serialization": inner,
        });
    case OperationalErrorUnknown:
        return json.Marshal("Unknown")
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
type OperatorDebug struct {}

func (OperatorDebug) isOperator() {}
type OperatorPrint struct {}

func (OperatorPrint) isOperator() {}
type OperatorCut struct {}

func (OperatorCut) isOperator() {}
type OperatorIn struct {}

func (OperatorIn) isOperator() {}
type OperatorIsa struct {}

func (OperatorIsa) isOperator() {}
type OperatorNew struct {}

func (OperatorNew) isOperator() {}
type OperatorDot struct {}

func (OperatorDot) isOperator() {}
type OperatorNot struct {}

func (OperatorNot) isOperator() {}
type OperatorMul struct {}

func (OperatorMul) isOperator() {}
type OperatorDiv struct {}

func (OperatorDiv) isOperator() {}
type OperatorMod struct {}

func (OperatorMod) isOperator() {}
type OperatorRem struct {}

func (OperatorRem) isOperator() {}
type OperatorAdd struct {}

func (OperatorAdd) isOperator() {}
type OperatorSub struct {}

func (OperatorSub) isOperator() {}
type OperatorEq struct {}

func (OperatorEq) isOperator() {}
type OperatorGeq struct {}

func (OperatorGeq) isOperator() {}
type OperatorLeq struct {}

func (OperatorLeq) isOperator() {}
type OperatorNeq struct {}

func (OperatorNeq) isOperator() {}
type OperatorGt struct {}

func (OperatorGt) isOperator() {}
type OperatorLt struct {}

func (OperatorLt) isOperator() {}
type OperatorUnify struct {}

func (OperatorUnify) isOperator() {}
type OperatorOr struct {}

func (OperatorOr) isOperator() {}
type OperatorAnd struct {}

func (OperatorAnd) isOperator() {}
type OperatorForAll struct {}

func (OperatorForAll) isOperator() {}
type OperatorAssign struct {}

func (OperatorAssign) isOperator() {}
// Operator enum
type OperatorVariant interface {
    isOperator()
}

type Operator struct {
    OperatorVariant
}

type OperatorDeserializer struct {
   Inner Operator
}

func (result *Operator) UnmarshalJSON(b []byte) error {
    var deserializer OperatorDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Print":
        var variant OperatorPrint
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Cut":
        var variant OperatorCut
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "In":
        var variant OperatorIn
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Isa":
        var variant OperatorIsa
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "New":
        var variant OperatorNew
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Dot":
        var variant OperatorDot
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Not":
        var variant OperatorNot
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Mul":
        var variant OperatorMul
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Div":
        var variant OperatorDiv
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Mod":
        var variant OperatorMod
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Rem":
        var variant OperatorRem
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Add":
        var variant OperatorAdd
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Sub":
        var variant OperatorSub
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Eq":
        var variant OperatorEq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Geq":
        var variant OperatorGeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Leq":
        var variant OperatorLeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Neq":
        var variant OperatorNeq
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Gt":
        var variant OperatorGt
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Lt":
        var variant OperatorLt
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Unify":
        var variant OperatorUnify
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Or":
        var variant OperatorOr
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "And":
        var variant OperatorAnd
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "ForAll":
        var variant OperatorForAll
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    case "Assign":
        var variant OperatorAssign
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = OperatorDeserializer { Inner: Operator { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Operator: %s", string(b))
}


func (variant Operator) MarshalJSON() ([]byte, error) {
    switch inner := variant.OperatorVariant.(type) {
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
type ParseErrorVariant interface {
    isParseError()
}

type ParseError struct {
    ParseErrorVariant
}

type ParseErrorDeserializer struct {
   Inner ParseError
}

func (result *ParseError) UnmarshalJSON(b []byte) error {
    var deserializer ParseErrorDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "InvalidTokenCharacter":
        var variant ParseErrorInvalidTokenCharacter
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "InvalidToken":
        var variant ParseErrorInvalidToken
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "UnrecognizedEOF":
        var variant ParseErrorUnrecognizedEOF
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "UnrecognizedToken":
        var variant ParseErrorUnrecognizedToken
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "ExtraToken":
        var variant ParseErrorExtraToken
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "ReservedWord":
        var variant ParseErrorReservedWord
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "InvalidFloat":
        var variant ParseErrorInvalidFloat
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "WrongValueType":
        var variant ParseErrorWrongValueType
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    case "DuplicateKey":
        var variant ParseErrorDuplicateKey
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ParseErrorDeserializer { Inner: ParseError { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize ParseError: %s", string(b))
}


func (variant ParseError) MarshalJSON() ([]byte, error) {
    switch inner := variant.ParseErrorVariant.(type) {
    case ParseErrorIntegerOverflow:
        return json.Marshal(map[string]ParseErrorIntegerOverflow {
            "IntegerOverflow": inner,
        });
    case ParseErrorInvalidTokenCharacter:
        return json.Marshal(map[string]ParseErrorInvalidTokenCharacter {
            "InvalidTokenCharacter": inner,
        });
    case ParseErrorInvalidToken:
        return json.Marshal(map[string]ParseErrorInvalidToken {
            "InvalidToken": inner,
        });
    case ParseErrorUnrecognizedEOF:
        return json.Marshal(map[string]ParseErrorUnrecognizedEOF {
            "UnrecognizedEOF": inner,
        });
    case ParseErrorUnrecognizedToken:
        return json.Marshal(map[string]ParseErrorUnrecognizedToken {
            "UnrecognizedToken": inner,
        });
    case ParseErrorExtraToken:
        return json.Marshal(map[string]ParseErrorExtraToken {
            "ExtraToken": inner,
        });
    case ParseErrorReservedWord:
        return json.Marshal(map[string]ParseErrorReservedWord {
            "ReservedWord": inner,
        });
    case ParseErrorInvalidFloat:
        return json.Marshal(map[string]ParseErrorInvalidFloat {
            "InvalidFloat": inner,
        });
    case ParseErrorWrongValueType:
        return json.Marshal(map[string]ParseErrorWrongValueType {
            "WrongValueType": inner,
        });
    case ParseErrorDuplicateKey:
        return json.Marshal(map[string]ParseErrorDuplicateKey {
            "DuplicateKey": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
// PatternDictionary newtype
type PatternDictionary Dictionary

func (variant PatternDictionary) MarshalJSON() ([]byte, error) {
    return json.Marshal(Dictionary(variant))
}

func (variant *PatternDictionary) UnmarshalJSON(b []byte) (error) {
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

func (variant *PatternInstance) UnmarshalJSON(b []byte) (error) {
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

type PatternDeserializer struct {
   Inner Pattern
}

func (result *Pattern) UnmarshalJSON(b []byte) error {
    var deserializer PatternDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = PatternDeserializer { Inner: Pattern { variant } }
        return nil
    case "Instance":
        var variant PatternInstance
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = PatternDeserializer { Inner: Pattern { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Pattern: %s", string(b))
}


func (variant Pattern) MarshalJSON() ([]byte, error) {
    switch inner := variant.PatternVariant.(type) {
    case PatternDictionary:
        return json.Marshal(map[string]PatternDictionary {
            "Dictionary": inner,
        });
    case PatternInstance:
        return json.Marshal(map[string]PatternInstance {
            "Instance": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
// Projection struct
type Projection struct {
    // 0
    V0 string
    // 1
    V1 *string
}

func (result *Projection) UnmarshalJSON(b []byte) error {
    var jsonFields []json.RawMessage
    json.Unmarshal(b, &jsonFields)

    if (len(jsonFields) != 2) {
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
type QueryEventNone struct {}

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

    *result = QueryEventExternalOp {
        CallId: intermediate.CallId,  
        Operator: intermediate.Operator.Inner,
        Args: intermediate.Args,  
    }
    return nil
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

type QueryEventDeserializer struct {
   Inner QueryEvent
}

func (result *QueryEvent) UnmarshalJSON(b []byte) error {
    var deserializer QueryEventDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "Done":
        var variant QueryEventDone
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "Debug":
        var variant QueryEventDebug
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "MakeExternal":
        var variant QueryEventMakeExternal
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "ExternalCall":
        var variant QueryEventExternalCall
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "ExternalIsa":
        var variant QueryEventExternalIsa
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "ExternalIsaWithPath":
        var variant QueryEventExternalIsaWithPath
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "ExternalIsSubSpecializer":
        var variant QueryEventExternalIsSubSpecializer
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "ExternalIsSubclass":
        var variant QueryEventExternalIsSubclass
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "Result":
        var variant QueryEventResult
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "ExternalOp":
        var variant QueryEventExternalOp
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    case "NextExternal":
        var variant QueryEventNextExternal
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = QueryEventDeserializer { Inner: QueryEvent { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize QueryEvent: %s", string(b))
}


func (variant QueryEvent) MarshalJSON() ([]byte, error) {
    switch inner := variant.QueryEventVariant.(type) {
    case QueryEventNone:
        return json.Marshal("None")
    case QueryEventDone:
        return json.Marshal(map[string]QueryEventDone {
            "Done": inner,
        });
    case QueryEventDebug:
        return json.Marshal(map[string]QueryEventDebug {
            "Debug": inner,
        });
    case QueryEventMakeExternal:
        return json.Marshal(map[string]QueryEventMakeExternal {
            "MakeExternal": inner,
        });
    case QueryEventExternalCall:
        return json.Marshal(map[string]QueryEventExternalCall {
            "ExternalCall": inner,
        });
    case QueryEventExternalIsa:
        return json.Marshal(map[string]QueryEventExternalIsa {
            "ExternalIsa": inner,
        });
    case QueryEventExternalIsaWithPath:
        return json.Marshal(map[string]QueryEventExternalIsaWithPath {
            "ExternalIsaWithPath": inner,
        });
    case QueryEventExternalIsSubSpecializer:
        return json.Marshal(map[string]QueryEventExternalIsSubSpecializer {
            "ExternalIsSubSpecializer": inner,
        });
    case QueryEventExternalIsSubclass:
        return json.Marshal(map[string]QueryEventExternalIsSubclass {
            "ExternalIsSubclass": inner,
        });
    case QueryEventResult:
        return json.Marshal(map[string]QueryEventResult {
            "Result": inner,
        });
    case QueryEventExternalOp:
        return json.Marshal(map[string]QueryEventExternalOp {
            "ExternalOp": inner,
        });
    case QueryEventNextExternal:
        return json.Marshal(map[string]QueryEventNextExternal {
            "NextExternal": inner,
        });
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
type Relation struct {
    // 0
    V0 string
    // 1
    V1 string
    // 2
    V2 string
}

func (result *Relation) UnmarshalJSON(b []byte) error {
    var jsonFields []json.RawMessage
    json.Unmarshal(b, &jsonFields)

    if (len(jsonFields) != 3) {
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
type RuntimeErrorMultipleLoadError struct {}

func (RuntimeErrorMultipleLoadError) isRuntimeError() {}
// RuntimeError enum
type RuntimeErrorVariant interface {
    isRuntimeError()
}

type RuntimeError struct {
    RuntimeErrorVariant
}

type RuntimeErrorDeserializer struct {
   Inner RuntimeError
}

func (result *RuntimeError) UnmarshalJSON(b []byte) error {
    var deserializer RuntimeErrorDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "Unsupported":
        var variant RuntimeErrorUnsupported
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "TypeError":
        var variant RuntimeErrorTypeError
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "StackOverflow":
        var variant RuntimeErrorStackOverflow
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "QueryTimeout":
        var variant RuntimeErrorQueryTimeout
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "Application":
        var variant RuntimeErrorApplication
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "IncompatibleBindings":
        var variant RuntimeErrorIncompatibleBindings
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "UnhandledPartial":
        var variant RuntimeErrorUnhandledPartial
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "DataFilteringFieldMissing":
        var variant RuntimeErrorDataFilteringFieldMissing
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "DataFilteringUnsupportedOp":
        var variant RuntimeErrorDataFilteringUnsupportedOp
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "InvalidRegistration":
        var variant RuntimeErrorInvalidRegistration
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "InvalidState":
        var variant RuntimeErrorInvalidState
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    case "MultipleLoadError":
        var variant RuntimeErrorMultipleLoadError
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = RuntimeErrorDeserializer { Inner: RuntimeError { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize RuntimeError: %s", string(b))
}


func (variant RuntimeError) MarshalJSON() ([]byte, error) {
    switch inner := variant.RuntimeErrorVariant.(type) {
    case RuntimeErrorArithmeticError:
        return json.Marshal(map[string]RuntimeErrorArithmeticError {
            "ArithmeticError": inner,
        });
    case RuntimeErrorUnsupported:
        return json.Marshal(map[string]RuntimeErrorUnsupported {
            "Unsupported": inner,
        });
    case RuntimeErrorTypeError:
        return json.Marshal(map[string]RuntimeErrorTypeError {
            "TypeError": inner,
        });
    case RuntimeErrorStackOverflow:
        return json.Marshal(map[string]RuntimeErrorStackOverflow {
            "StackOverflow": inner,
        });
    case RuntimeErrorQueryTimeout:
        return json.Marshal(map[string]RuntimeErrorQueryTimeout {
            "QueryTimeout": inner,
        });
    case RuntimeErrorApplication:
        return json.Marshal(map[string]RuntimeErrorApplication {
            "Application": inner,
        });
    case RuntimeErrorIncompatibleBindings:
        return json.Marshal(map[string]RuntimeErrorIncompatibleBindings {
            "IncompatibleBindings": inner,
        });
    case RuntimeErrorUnhandledPartial:
        return json.Marshal(map[string]RuntimeErrorUnhandledPartial {
            "UnhandledPartial": inner,
        });
    case RuntimeErrorDataFilteringFieldMissing:
        return json.Marshal(map[string]RuntimeErrorDataFilteringFieldMissing {
            "DataFilteringFieldMissing": inner,
        });
    case RuntimeErrorDataFilteringUnsupportedOp:
        return json.Marshal(map[string]RuntimeErrorDataFilteringUnsupportedOp {
            "DataFilteringUnsupportedOp": inner,
        });
    case RuntimeErrorInvalidRegistration:
        return json.Marshal(map[string]RuntimeErrorInvalidRegistration {
            "InvalidRegistration": inner,
        });
    case RuntimeErrorInvalidState:
        return json.Marshal(map[string]RuntimeErrorInvalidState {
            "InvalidState": inner,
        });
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

// Symbol newtype
type Symbol string

func (variant Symbol) MarshalJSON() ([]byte, error) {
    return json.Marshal(string(variant))
}

func (variant *Symbol) UnmarshalJSON(b []byte) (error) {
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

func (result *Term) UnmarshalJSON(b []byte) error {

    type RawTerm struct {
        // Value
        Value ValueDeserializer `json:"value"`
    }
    var intermediate RawTerm
    err := json.Unmarshal(b, &intermediate)
    if err != nil {
        return err
    }

    *result = Term {
        Value: intermediate.Value.Inner,
    }
    return nil
}
// Trace struct
type Trace struct {
    // Node
    Node Node `json:"node"`
    // Children
    Children []Trace `json:"children"`
}

func (result *Trace) UnmarshalJSON(b []byte) error {

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

    *result = Trace {
        Node: intermediate.Node.Inner,
        Children: intermediate.Children,  
    }
    return nil
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
type TypeVariant interface {
    isType()
}

type Type struct {
    TypeVariant
}

type TypeDeserializer struct {
   Inner Type
}

func (result *Type) UnmarshalJSON(b []byte) error {
    var deserializer TypeDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = TypeDeserializer { Inner: Type { variant } }
        return nil
    case "Relation":
        var variant TypeRelation
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = TypeDeserializer { Inner: Type { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Type: %s", string(b))
}


func (variant Type) MarshalJSON() ([]byte, error) {
    switch inner := variant.TypeVariant.(type) {
    case TypeBase:
        return json.Marshal(map[string]TypeBase {
            "Base": inner,
        });
    case TypeRelation:
        return json.Marshal(map[string]TypeRelation {
            "Relation": inner,
        });
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

    *result = ValidationErrorDuplicateResourceBlockDeclaration {
        Resource: intermediate.Resource,  
        Declaration: intermediate.Declaration,  
        Existing: intermediate.Existing.Inner,
        New: intermediate.New.Inner,
    }
    return nil
}

func (ValidationErrorDuplicateResourceBlockDeclaration) isValidationError() {}
// ValidationError enum
type ValidationErrorVariant interface {
    isValidationError()
}

type ValidationError struct {
    ValidationErrorVariant
}

type ValidationErrorDeserializer struct {
   Inner ValidationError
}

func (result *ValidationError) UnmarshalJSON(b []byte) error {
    var deserializer ValidationErrorDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "MissingRequiredRule":
        var variant ValidationErrorMissingRequiredRule
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "InvalidRule":
        var variant ValidationErrorInvalidRule
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "InvalidRuleType":
        var variant ValidationErrorInvalidRuleType
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "UndefinedRuleCall":
        var variant ValidationErrorUndefinedRuleCall
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "ResourceBlock":
        var variant ValidationErrorResourceBlock
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "SingletonVariable":
        var variant ValidationErrorSingletonVariable
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "UnregisteredClass":
        var variant ValidationErrorUnregisteredClass
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    case "DuplicateResourceBlockDeclaration":
        var variant ValidationErrorDuplicateResourceBlockDeclaration
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValidationErrorDeserializer { Inner: ValidationError { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize ValidationError: %s", string(b))
}


func (variant ValidationError) MarshalJSON() ([]byte, error) {
    switch inner := variant.ValidationErrorVariant.(type) {
    case ValidationErrorFileLoading:
        return json.Marshal(map[string]ValidationErrorFileLoading {
            "FileLoading": inner,
        });
    case ValidationErrorMissingRequiredRule:
        return json.Marshal(map[string]ValidationErrorMissingRequiredRule {
            "MissingRequiredRule": inner,
        });
    case ValidationErrorInvalidRule:
        return json.Marshal(map[string]ValidationErrorInvalidRule {
            "InvalidRule": inner,
        });
    case ValidationErrorInvalidRuleType:
        return json.Marshal(map[string]ValidationErrorInvalidRuleType {
            "InvalidRuleType": inner,
        });
    case ValidationErrorUndefinedRuleCall:
        return json.Marshal(map[string]ValidationErrorUndefinedRuleCall {
            "UndefinedRuleCall": inner,
        });
    case ValidationErrorResourceBlock:
        return json.Marshal(map[string]ValidationErrorResourceBlock {
            "ResourceBlock": inner,
        });
    case ValidationErrorSingletonVariable:
        return json.Marshal(map[string]ValidationErrorSingletonVariable {
            "SingletonVariable": inner,
        });
    case ValidationErrorUnregisteredClass:
        return json.Marshal(map[string]ValidationErrorUnregisteredClass {
            "UnregisteredClass": inner,
        });
    case ValidationErrorDuplicateResourceBlockDeclaration:
        return json.Marshal(map[string]ValidationErrorDuplicateResourceBlockDeclaration {
            "DuplicateResourceBlockDeclaration": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
// ValueNumber newtype
type ValueNumber Numeric

func (variant ValueNumber) MarshalJSON() ([]byte, error) {
    return json.Marshal(Numeric(variant))
}

func (variant *ValueNumber) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueString) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueBoolean) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueExternalInstance) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueDictionary) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValuePattern) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueCall) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueList) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueVariable) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueRestVariable) UnmarshalJSON(b []byte) (error) {
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

func (variant *ValueExpression) UnmarshalJSON(b []byte) (error) {
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

type ValueDeserializer struct {
   Inner Value
}

func (result *Value) UnmarshalJSON(b []byte) error {
    var deserializer ValueDeserializer
    err := json.Unmarshal(b, &deserializer)
    if err != nil {
        return err
    }
    *result = deserializer.Inner
    return nil
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
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "String":
        var variant ValueString
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "Boolean":
        var variant ValueBoolean
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "ExternalInstance":
        var variant ValueExternalInstance
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "Dictionary":
        var variant ValueDictionary
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "Pattern":
        var variant ValuePattern
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "Call":
        var variant ValueCall
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "List":
        var variant ValueList
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "Variable":
        var variant ValueVariable
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "RestVariable":
        var variant ValueRestVariable
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    case "Expression":
        var variant ValueExpression
        if variantValue != nil {
            err := json.Unmarshal(*variantValue, &variant);
            if err != nil {
                return err
            }
        }
        *result = ValueDeserializer { Inner: Value { variant } }
        return nil
    }

    return fmt.Errorf("cannot deserialize Value: %s", string(b))
}


func (variant Value) MarshalJSON() ([]byte, error) {
    switch inner := variant.ValueVariant.(type) {
    case ValueNumber:
        return json.Marshal(map[string]ValueNumber {
            "Number": inner,
        });
    case ValueString:
        return json.Marshal(map[string]ValueString {
            "String": inner,
        });
    case ValueBoolean:
        return json.Marshal(map[string]ValueBoolean {
            "Boolean": inner,
        });
    case ValueExternalInstance:
        return json.Marshal(map[string]ValueExternalInstance {
            "ExternalInstance": inner,
        });
    case ValueDictionary:
        return json.Marshal(map[string]ValueDictionary {
            "Dictionary": inner,
        });
    case ValuePattern:
        return json.Marshal(map[string]ValuePattern {
            "Pattern": inner,
        });
    case ValueCall:
        return json.Marshal(map[string]ValueCall {
            "Call": inner,
        });
    case ValueList:
        return json.Marshal(map[string]ValueList {
            "List": inner,
        });
    case ValueVariable:
        return json.Marshal(map[string]ValueVariable {
            "Variable": inner,
        });
    case ValueRestVariable:
        return json.Marshal(map[string]ValueRestVariable {
            "RestVariable": inner,
        });
    case ValueExpression:
        return json.Marshal(map[string]ValueExpression {
            "Expression": inner,
        });
    default:
        return nil, fmt.Errorf("unexpected variant %#v of %v", inner, variant)
    }

}
