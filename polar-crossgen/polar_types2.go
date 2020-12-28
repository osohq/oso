package oso

import (
    "encoding/json"
    "errors"
)


// Call struct
type Call struct {
    // Name
    Name string
    // Args
    Args []Value
    // Kwargs
    Kwargs *map[string]Value
}


// Dictionary struct
type Dictionary struct {
    // Fields
    Fields map[string]Value
}


// ExternalInstance struct
type ExternalInstance struct {
    // InstanceId
    InstanceId uint64
    // Constructor
    Constructor *Value
    // Repr
    Repr *string
}


// InstanceLiteral struct
type InstanceLiteral struct {
    // Tag
    Tag string
    // Fields
    Fields Dictionary
}


// Node enum
type NodeVariant interface {
    isNode()
}

type Node struct {
    *NodeVariant
}


func (v *Node) json.UnmarshalJson(b []byte) error {
    var result Node
    var rawMap map[string]json.RawMessage
    
    err := json.Unmarshal(j, &rawMap)
    if err != nil { return result, err }

    if len(rawMap) != 1 {
        return result, errors.New("Deserializing Node as an enum variant; expecting a single key")
    }

    for k, v := range rawMap {
        switch k {
        
        case "Rule":
            if val, err := deserializeRule(v); err == nil {
                *v = Node { *val }
                return nil
            } else {
                return err
            }
        
        case "Term":
            if val, err := deserializeTerm(v); err == nil {
                *v = Node { *val }
                return nil
            } else {
                return err
            }
        
        default:
            return fmt.Errorf("Unknown variant for Node: %s", k)
        }
    }
    return fmt.Printf("unreachable")
}
// NodeRule newtype
type NodeRule Rule




func (*NodeRule) isNode() {}
// NodeTerm newtype
type NodeTerm Value




func (*NodeTerm) isNode() {}
// Numeric enum
type NumericVariant interface {
    isNumeric()
}

type Numeric struct {
    *NumericVariant
}


func (v *Numeric) json.UnmarshalJson(b []byte) error {
    var result Numeric
    var rawMap map[string]json.RawMessage
    
    err := json.Unmarshal(j, &rawMap)
    if err != nil { return result, err }

    if len(rawMap) != 1 {
        return result, errors.New("Deserializing Numeric as an enum variant; expecting a single key")
    }

    for k, v := range rawMap {
        switch k {
        
        case "Integer":
            if val, err := deserializeInteger(v); err == nil {
                *v = Numeric { *val }
                return nil
            } else {
                return err
            }
        
        case "Float":
            if val, err := deserializeFloat(v); err == nil {
                *v = Numeric { *val }
                return nil
            } else {
                return err
            }
        
        default:
            return fmt.Errorf("Unknown variant for Numeric: %s", k)
        }
    }
    return fmt.Printf("unreachable")
}
// NumericInteger newtype
type NumericInteger int64




func (*NumericInteger) isNumeric() {}
// NumericFloat newtype
type NumericFloat float64




func (*NumericFloat) isNumeric() {}
// Operation struct
type Operation struct {
    // Operator
    Operator Operator
    // Args
    Args []Value
}


// Operator enum
type OperatorVariant interface {
    isOperator()
}

type Operator struct {
    *OperatorVariant
}


func (v *Operator) json.UnmarshalJson(b []byte) error {
    var result Operator
    var rawMap map[string]json.RawMessage
    
    err := json.Unmarshal(j, &rawMap)
    if err != nil { return result, err }

    if len(rawMap) != 1 {
        return result, errors.New("Deserializing Operator as an enum variant; expecting a single key")
    }

    for k, v := range rawMap {
        switch k {
        
        case "Debug":
            if val, err := deserializeDebug(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Print":
            if val, err := deserializePrint(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Cut":
            if val, err := deserializeCut(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "In":
            if val, err := deserializeIn(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Isa":
            if val, err := deserializeIsa(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "New":
            if val, err := deserializeNew(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Dot":
            if val, err := deserializeDot(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Not":
            if val, err := deserializeNot(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Mul":
            if val, err := deserializeMul(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Div":
            if val, err := deserializeDiv(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Mod":
            if val, err := deserializeMod(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Rem":
            if val, err := deserializeRem(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Add":
            if val, err := deserializeAdd(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Sub":
            if val, err := deserializeSub(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Eq":
            if val, err := deserializeEq(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Geq":
            if val, err := deserializeGeq(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Leq":
            if val, err := deserializeLeq(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Neq":
            if val, err := deserializeNeq(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Gt":
            if val, err := deserializeGt(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Lt":
            if val, err := deserializeLt(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Unify":
            if val, err := deserializeUnify(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Or":
            if val, err := deserializeOr(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "And":
            if val, err := deserializeAnd(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "ForAll":
            if val, err := deserializeForAll(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        case "Assign":
            if val, err := deserializeAssign(v); err == nil {
                *v = Operator { *val }
                return nil
            } else {
                return err
            }
        
        default:
            return fmt.Errorf("Unknown variant for Operator: %s", k)
        }
    }
    return fmt.Printf("unreachable")
}
type Operator__Debug struct {}

type Operator__Print struct {}

type Operator__Cut struct {}

type Operator__In struct {}

type Operator__Isa struct {}

type Operator__New struct {}

type Operator__Dot struct {}

type Operator__Not struct {}

type Operator__Mul struct {}

type Operator__Div struct {}

type Operator__Mod struct {}

type Operator__Rem struct {}

type Operator__Add struct {}

type Operator__Sub struct {}

type Operator__Eq struct {}

type Operator__Geq struct {}

type Operator__Leq struct {}

type Operator__Neq struct {}

type Operator__Gt struct {}

type Operator__Lt struct {}

type Operator__Unify struct {}

type Operator__Or struct {}

type Operator__And struct {}

type Operator__ForAll struct {}

type Operator__Assign struct {}

// Parameter struct
type Parameter struct {
    // Parameter
    Parameter Value
    // Specializer
    Specializer *Value
}


// Partial struct
type Partial struct {
    // Constraints
    Constraints []Operation
    // Variable
    Variable string
}


// Pattern enum
type PatternVariant interface {
    isPattern()
}

type Pattern struct {
    *PatternVariant
}


func (v *Pattern) json.UnmarshalJson(b []byte) error {
    var result Pattern
    var rawMap map[string]json.RawMessage
    
    err := json.Unmarshal(j, &rawMap)
    if err != nil { return result, err }

    if len(rawMap) != 1 {
        return result, errors.New("Deserializing Pattern as an enum variant; expecting a single key")
    }

    for k, v := range rawMap {
        switch k {
        
        case "Dictionary":
            if val, err := deserializeDictionary(v); err == nil {
                *v = Pattern { *val }
                return nil
            } else {
                return err
            }
        
        case "Instance":
            if val, err := deserializeInstance(v); err == nil {
                *v = Pattern { *val }
                return nil
            } else {
                return err
            }
        
        default:
            return fmt.Errorf("Unknown variant for Pattern: %s", k)
        }
    }
    return fmt.Printf("unreachable")
}
// PatternDictionary newtype
type PatternDictionary Dictionary




func (*PatternDictionary) isPattern() {}
// PatternInstance newtype
type PatternInstance InstanceLiteral




func (*PatternInstance) isPattern() {}
// QueryEvent enum
type QueryEventVariant interface {
    isQueryEvent()
}

type QueryEvent struct {
    *QueryEventVariant
}


func (v *QueryEvent) json.UnmarshalJson(b []byte) error {
    var result QueryEvent
    var rawMap map[string]json.RawMessage
    
    err := json.Unmarshal(j, &rawMap)
    if err != nil { return result, err }

    if len(rawMap) != 1 {
        return result, errors.New("Deserializing QueryEvent as an enum variant; expecting a single key")
    }

    for k, v := range rawMap {
        switch k {
        
        case "None":
            if val, err := deserializeNone(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "Done":
            if val, err := deserializeDone(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "Debug":
            if val, err := deserializeDebug(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "MakeExternal":
            if val, err := deserializeMakeExternal(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalCall":
            if val, err := deserializeExternalCall(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalIsa":
            if val, err := deserializeExternalIsa(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalIsSubSpecializer":
            if val, err := deserializeExternalIsSubSpecializer(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalIsSubclass":
            if val, err := deserializeExternalIsSubclass(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalUnify":
            if val, err := deserializeExternalUnify(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "Result":
            if val, err := deserializeResult(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalOp":
            if val, err := deserializeExternalOp(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        case "NextExternal":
            if val, err := deserializeNextExternal(v); err == nil {
                *v = QueryEvent { *val }
                return nil
            } else {
                return err
            }
        
        default:
            return fmt.Errorf("Unknown variant for QueryEvent: %s", k)
        }
    }
    return fmt.Printf("unreachable")
}
type QueryEvent__None struct {}

// QueryEventDone struct
type QueryEventDone struct {
    // Result
    Result bool
}




func (*QueryEventDone) isQueryEvent() {}
// QueryEventDebug struct
type QueryEventDebug struct {
    // Message
    Message string
}




func (*QueryEventDebug) isQueryEvent() {}
// QueryEventMakeExternal struct
type QueryEventMakeExternal struct {
    // InstanceId
    InstanceId uint64
    // Constructor
    Constructor Value
}




func (*QueryEventMakeExternal) isQueryEvent() {}
// QueryEventExternalCall struct
type QueryEventExternalCall struct {
    // CallId
    CallId uint64
    // Instance
    Instance Value
    // Attribute
    Attribute string
    // Args
    Args *[]Value
    // Kwargs
    Kwargs *map[string]Value
}




func (*QueryEventExternalCall) isQueryEvent() {}
// QueryEventExternalIsa struct
type QueryEventExternalIsa struct {
    // CallId
    CallId uint64
    // Instance
    Instance Value
    // ClassTag
    ClassTag string
}




func (*QueryEventExternalIsa) isQueryEvent() {}
// QueryEventExternalIsSubSpecializer struct
type QueryEventExternalIsSubSpecializer struct {
    // CallId
    CallId uint64
    // InstanceId
    InstanceId uint64
    // LeftClassTag
    LeftClassTag string
    // RightClassTag
    RightClassTag string
}




func (*QueryEventExternalIsSubSpecializer) isQueryEvent() {}
// QueryEventExternalIsSubclass struct
type QueryEventExternalIsSubclass struct {
    // CallId
    CallId uint64
    // LeftClassTag
    LeftClassTag string
    // RightClassTag
    RightClassTag string
}




func (*QueryEventExternalIsSubclass) isQueryEvent() {}
// QueryEventExternalUnify struct
type QueryEventExternalUnify struct {
    // CallId
    CallId uint64
    // LeftInstanceId
    LeftInstanceId uint64
    // RightInstanceId
    RightInstanceId uint64
}




func (*QueryEventExternalUnify) isQueryEvent() {}
// QueryEventResult struct
type QueryEventResult struct {
    // Bindings
    Bindings map[string]Value
    // Trace
    Trace *TraceResult
}




func (*QueryEventResult) isQueryEvent() {}
// QueryEventExternalOp struct
type QueryEventExternalOp struct {
    // CallId
    CallId uint64
    // Operator
    Operator Operator
    // Args
    Args []Value
}




func (*QueryEventExternalOp) isQueryEvent() {}
// QueryEventNextExternal struct
type QueryEventNextExternal struct {
    // CallId
    CallId uint64
    // Iterable
    Iterable Value
}




func (*QueryEventNextExternal) isQueryEvent() {}
// Rule struct
type Rule struct {
    // Name
    Name string
    // Params
    Params []Parameter
    // Body
    Body Value
}


// Trace struct
type Trace struct {
    // Node
    Node Node
    // Children
    Children []Trace
}


// TraceResult struct
type TraceResult struct {
    // Trace
    Trace Trace
    // Formatted
    Formatted string
}


// Value enum
type ValueVariant interface {
    isValue()
}

type Value struct {
    *ValueVariant
}


func (v *Value) json.UnmarshalJson(b []byte) error {
    var result Value
    var rawMap map[string]json.RawMessage
    
    err := json.Unmarshal(j, &rawMap)
    if err != nil { return result, err }

    if len(rawMap) != 1 {
        return result, errors.New("Deserializing Value as an enum variant; expecting a single key")
    }

    for k, v := range rawMap {
        switch k {
        
        case "Number":
            if val, err := deserializeNumber(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "String":
            if val, err := deserializeString(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Boolean":
            if val, err := deserializeBoolean(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "ExternalInstance":
            if val, err := deserializeExternalInstance(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "InstanceLiteral":
            if val, err := deserializeInstanceLiteral(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Dictionary":
            if val, err := deserializeDictionary(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Pattern":
            if val, err := deserializePattern(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Call":
            if val, err := deserializeCall(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "List":
            if val, err := deserializeList(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Variable":
            if val, err := deserializeVariable(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "RestVariable":
            if val, err := deserializeRestVariable(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Expression":
            if val, err := deserializeExpression(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        case "Partial":
            if val, err := deserializePartial(v); err == nil {
                *v = Value { *val }
                return nil
            } else {
                return err
            }
        
        default:
            return fmt.Errorf("Unknown variant for Value: %s", k)
        }
    }
    return fmt.Printf("unreachable")
}
// ValueNumber newtype
type ValueNumber Numeric




func (*ValueNumber) isValue() {}
// ValueString newtype
type ValueString string




func (*ValueString) isValue() {}
// ValueBoolean newtype
type ValueBoolean bool




func (*ValueBoolean) isValue() {}
// ValueExternalInstance newtype
type ValueExternalInstance ExternalInstance




func (*ValueExternalInstance) isValue() {}
// ValueInstanceLiteral newtype
type ValueInstanceLiteral InstanceLiteral




func (*ValueInstanceLiteral) isValue() {}
// ValueDictionary newtype
type ValueDictionary Dictionary




func (*ValueDictionary) isValue() {}
// ValuePattern newtype
type ValuePattern Pattern




func (*ValuePattern) isValue() {}
// ValueCall newtype
type ValueCall Call




func (*ValueCall) isValue() {}
// ValueList newtype
type ValueList []Value




func (*ValueList) isValue() {}
// ValueVariable newtype
type ValueVariable string




func (*ValueVariable) isValue() {}
// ValueRestVariable newtype
type ValueRestVariable string




func (*ValueRestVariable) isValue() {}
// ValueExpression newtype
type ValueExpression Operation




func (*ValueExpression) isValue() {}
// ValuePartial newtype
type ValuePartial Partial




func (*ValuePartial) isValue() {}
