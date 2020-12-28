package oso

import (
	"fmt"
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/serde"
)

type Call struct {
	Name   string
	Args   []Value
	Kwargs *map[string]Value
}

func (obj *Call) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.Name); err != nil {
		return err
	}
	if err := serialize_vector_Value(obj.Args, serializer); err != nil {
		return err
	}
	if err := serialize_option_map_str_to_Value(obj.Kwargs, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeCall(deserializer serde.Deserializer) (Call, error) {
	var obj Call
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Name = val
	} else {
		return obj, err
	}
	if val, err := deserialize_vector_Value(deserializer); err == nil {
		obj.Args = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_map_str_to_Value(deserializer); err == nil {
		obj.Kwargs = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Dictionary struct {
	Fields map[string]Value
}

func (obj *Dictionary) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := serialize_map_str_to_Value(obj.Fields, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeDictionary(deserializer serde.Deserializer) (Dictionary, error) {
	var obj Dictionary
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserialize_map_str_to_Value(deserializer); err == nil {
		obj.Fields = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type ExternalInstance struct {
	InstanceId  uint64
	Constructor *Value
	Repr        *string
}

func (obj *ExternalInstance) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := serializer.SerializeU64(obj.InstanceId); err != nil {
		return err
	}
	if err := serialize_option_Value(obj.Constructor, serializer); err != nil {
		return err
	}
	if err := serialize_option_str(obj.Repr, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeExternalInstance(deserializer serde.Deserializer) (ExternalInstance, error) {
	var obj ExternalInstance
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.InstanceId = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_Value(deserializer); err == nil {
		obj.Constructor = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_str(deserializer); err == nil {
		obj.Repr = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type InstanceLiteral struct {
	Tag    string
	Fields Dictionary
}

func (obj *InstanceLiteral) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.Tag); err != nil {
		return err
	}
	if err := obj.Fields.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeInstanceLiteral(deserializer serde.Deserializer) (InstanceLiteral, error) {
	var obj InstanceLiteral
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Tag = val
	} else {
		return obj, err
	}
	if val, err := DeserializeDictionary(deserializer); err == nil {
		obj.Fields = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Node interface {
	isNode()
	Serialize(serializer serde.Serializer) error
}

func DeserializeNode(deserializer serde.Deserializer) (Node, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil {
		return nil, err
	}

	switch index {
	case 0:
		if val, err := load_Node__Rule(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Node__Term(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Node: %d", index)
	}
}

type Node__Rule struct {
	Value Rule
}

func (*Node__Rule) isNode() {}

func (obj *Node__Rule) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(0)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Node__Rule(deserializer serde.Deserializer) (Node__Rule, error) {
	var obj Node__Rule
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeRule(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Node__Term struct {
	Value Value
}

func (*Node__Term) isNode() {}

func (obj *Node__Term) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(1)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Node__Term(deserializer serde.Deserializer) (Node__Term, error) {
	var obj Node__Term
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Numeric interface {
	isNumeric()
	Serialize(serializer serde.Serializer) error
}

func DeserializeNumeric(deserializer serde.Deserializer) (Numeric, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil {
		return nil, err
	}

	switch index {
	case 0:
		if val, err := load_Numeric__Integer(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Numeric__Float(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Numeric: %d", index)
	}
}

type Numeric__Integer int64

func (*Numeric__Integer) isNumeric() {}

func (obj *Numeric__Integer) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(0)
	if err := serializer.SerializeI64(((int64)(*obj))); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Numeric__Integer(deserializer serde.Deserializer) (Numeric__Integer, error) {
	var obj int64
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Numeric__Integer)(obj), err
	}
	if val, err := deserializer.DeserializeI64(); err == nil {
		obj = val
	} else {
		return ((Numeric__Integer)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Numeric__Integer)(obj), nil
}

type Numeric__Float float64

func (*Numeric__Float) isNumeric() {}

func (obj *Numeric__Float) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeF64(((float64)(*obj))); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Numeric__Float(deserializer serde.Deserializer) (Numeric__Float, error) {
	var obj float64
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Numeric__Float)(obj), err
	}
	if val, err := deserializer.DeserializeF64(); err == nil {
		obj = val
	} else {
		return ((Numeric__Float)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Numeric__Float)(obj), nil
}

type Operation struct {
	Operator Operator
	Args     []Value
}

func (obj *Operation) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := obj.Operator.Serialize(serializer); err != nil {
		return err
	}
	if err := serialize_vector_Value(obj.Args, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeOperation(deserializer serde.Deserializer) (Operation, error) {
	var obj Operation
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeOperator(deserializer); err == nil {
		obj.Operator = val
	} else {
		return obj, err
	}
	if val, err := deserialize_vector_Value(deserializer); err == nil {
		obj.Args = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator interface {
	isOperator()
	Serialize(serializer serde.Serializer) error
}

func DeserializeOperator(deserializer serde.Deserializer) (Operator, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil {
		return nil, err
	}

	switch index {
	case 0:
		if val, err := load_Operator__Debug(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Operator__Print(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_Operator__Cut(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_Operator__In(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 4:
		if val, err := load_Operator__Isa(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 5:
		if val, err := load_Operator__New(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 6:
		if val, err := load_Operator__Dot(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 7:
		if val, err := load_Operator__Not(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 8:
		if val, err := load_Operator__Mul(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 9:
		if val, err := load_Operator__Div(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 10:
		if val, err := load_Operator__Mod(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 11:
		if val, err := load_Operator__Rem(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 12:
		if val, err := load_Operator__Add(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 13:
		if val, err := load_Operator__Sub(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 14:
		if val, err := load_Operator__Eq(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 15:
		if val, err := load_Operator__Geq(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 16:
		if val, err := load_Operator__Leq(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 17:
		if val, err := load_Operator__Neq(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 18:
		if val, err := load_Operator__Gt(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 19:
		if val, err := load_Operator__Lt(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 20:
		if val, err := load_Operator__Unify(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 21:
		if val, err := load_Operator__Or(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 22:
		if val, err := load_Operator__And(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 23:
		if val, err := load_Operator__ForAll(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 24:
		if val, err := load_Operator__Assign(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Operator: %d", index)
	}
}

type Operator__Debug struct {
}

func (*Operator__Debug) isOperator() {}

func (obj *Operator__Debug) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Debug(deserializer serde.Deserializer) (Operator__Debug, error) {
	var obj Operator__Debug
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Print struct {
}

func (*Operator__Print) isOperator() {}

func (obj *Operator__Print) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(1)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Print(deserializer serde.Deserializer) (Operator__Print, error) {
	var obj Operator__Print
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Cut struct {
}

func (*Operator__Cut) isOperator() {}

func (obj *Operator__Cut) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(2)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Cut(deserializer serde.Deserializer) (Operator__Cut, error) {
	var obj Operator__Cut
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__In struct {
}

func (*Operator__In) isOperator() {}

func (obj *Operator__In) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(3)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__In(deserializer serde.Deserializer) (Operator__In, error) {
	var obj Operator__In
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Isa struct {
}

func (*Operator__Isa) isOperator() {}

func (obj *Operator__Isa) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(4)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Isa(deserializer serde.Deserializer) (Operator__Isa, error) {
	var obj Operator__Isa
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__New struct {
}

func (*Operator__New) isOperator() {}

func (obj *Operator__New) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(5)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__New(deserializer serde.Deserializer) (Operator__New, error) {
	var obj Operator__New
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Dot struct {
}

func (*Operator__Dot) isOperator() {}

func (obj *Operator__Dot) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(6)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Dot(deserializer serde.Deserializer) (Operator__Dot, error) {
	var obj Operator__Dot
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Not struct {
}

func (*Operator__Not) isOperator() {}

func (obj *Operator__Not) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(7)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Not(deserializer serde.Deserializer) (Operator__Not, error) {
	var obj Operator__Not
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Mul struct {
}

func (*Operator__Mul) isOperator() {}

func (obj *Operator__Mul) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(8)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Mul(deserializer serde.Deserializer) (Operator__Mul, error) {
	var obj Operator__Mul
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Div struct {
}

func (*Operator__Div) isOperator() {}

func (obj *Operator__Div) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(9)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Div(deserializer serde.Deserializer) (Operator__Div, error) {
	var obj Operator__Div
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Mod struct {
}

func (*Operator__Mod) isOperator() {}

func (obj *Operator__Mod) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(10)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Mod(deserializer serde.Deserializer) (Operator__Mod, error) {
	var obj Operator__Mod
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Rem struct {
}

func (*Operator__Rem) isOperator() {}

func (obj *Operator__Rem) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(11)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Rem(deserializer serde.Deserializer) (Operator__Rem, error) {
	var obj Operator__Rem
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Add struct {
}

func (*Operator__Add) isOperator() {}

func (obj *Operator__Add) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(12)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Add(deserializer serde.Deserializer) (Operator__Add, error) {
	var obj Operator__Add
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Sub struct {
}

func (*Operator__Sub) isOperator() {}

func (obj *Operator__Sub) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(13)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Sub(deserializer serde.Deserializer) (Operator__Sub, error) {
	var obj Operator__Sub
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Eq struct {
}

func (*Operator__Eq) isOperator() {}

func (obj *Operator__Eq) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(14)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Eq(deserializer serde.Deserializer) (Operator__Eq, error) {
	var obj Operator__Eq
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Geq struct {
}

func (*Operator__Geq) isOperator() {}

func (obj *Operator__Geq) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(15)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Geq(deserializer serde.Deserializer) (Operator__Geq, error) {
	var obj Operator__Geq
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Leq struct {
}

func (*Operator__Leq) isOperator() {}

func (obj *Operator__Leq) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(16)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Leq(deserializer serde.Deserializer) (Operator__Leq, error) {
	var obj Operator__Leq
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Neq struct {
}

func (*Operator__Neq) isOperator() {}

func (obj *Operator__Neq) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(17)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Neq(deserializer serde.Deserializer) (Operator__Neq, error) {
	var obj Operator__Neq
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Gt struct {
}

func (*Operator__Gt) isOperator() {}

func (obj *Operator__Gt) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(18)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Gt(deserializer serde.Deserializer) (Operator__Gt, error) {
	var obj Operator__Gt
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Lt struct {
}

func (*Operator__Lt) isOperator() {}

func (obj *Operator__Lt) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(19)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Lt(deserializer serde.Deserializer) (Operator__Lt, error) {
	var obj Operator__Lt
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Unify struct {
}

func (*Operator__Unify) isOperator() {}

func (obj *Operator__Unify) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(20)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Unify(deserializer serde.Deserializer) (Operator__Unify, error) {
	var obj Operator__Unify
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Or struct {
}

func (*Operator__Or) isOperator() {}

func (obj *Operator__Or) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(21)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Or(deserializer serde.Deserializer) (Operator__Or, error) {
	var obj Operator__Or
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__And struct {
}

func (*Operator__And) isOperator() {}

func (obj *Operator__And) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(22)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__And(deserializer serde.Deserializer) (Operator__And, error) {
	var obj Operator__And
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__ForAll struct {
}

func (*Operator__ForAll) isOperator() {}

func (obj *Operator__ForAll) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(23)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__ForAll(deserializer serde.Deserializer) (Operator__ForAll, error) {
	var obj Operator__ForAll
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Operator__Assign struct {
}

func (*Operator__Assign) isOperator() {}

func (obj *Operator__Assign) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(24)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Operator__Assign(deserializer serde.Deserializer) (Operator__Assign, error) {
	var obj Operator__Assign
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Parameter struct {
	Parameter   Value
	Specializer *Value
}

func (obj *Parameter) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := obj.Parameter.Serialize(serializer); err != nil {
		return err
	}
	if err := serialize_option_Value(obj.Specializer, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeParameter(deserializer serde.Deserializer) (Parameter, error) {
	var obj Parameter
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Parameter = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_Value(deserializer); err == nil {
		obj.Specializer = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Partial struct {
	Constraints []Operation
	Variable    string
}

func (obj *Partial) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := serialize_vector_Operation(obj.Constraints, serializer); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.Variable); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializePartial(deserializer serde.Deserializer) (Partial, error) {
	var obj Partial
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserialize_vector_Operation(deserializer); err == nil {
		obj.Constraints = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Variable = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Pattern interface {
	isPattern()
	Serialize(serializer serde.Serializer) error
}

func DeserializePattern(deserializer serde.Deserializer) (Pattern, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil {
		return nil, err
	}

	switch index {
	case 0:
		if val, err := load_Pattern__Dictionary(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Pattern__Instance(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Pattern: %d", index)
	}
}

type Pattern__Dictionary struct {
	Value Dictionary
}

func (*Pattern__Dictionary) isPattern() {}

func (obj *Pattern__Dictionary) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(0)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Pattern__Dictionary(deserializer serde.Deserializer) (Pattern__Dictionary, error) {
	var obj Pattern__Dictionary
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeDictionary(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Pattern__Instance struct {
	Value InstanceLiteral
}

func (*Pattern__Instance) isPattern() {}

func (obj *Pattern__Instance) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(1)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Pattern__Instance(deserializer serde.Deserializer) (Pattern__Instance, error) {
	var obj Pattern__Instance
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeInstanceLiteral(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent interface {
	isQueryEvent()
	Serialize(serializer serde.Serializer) error
}

func DeserializeQueryEvent(deserializer serde.Deserializer) (QueryEvent, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil {
		return nil, err
	}

	switch index {
	case 0:
		if val, err := load_QueryEvent__None(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_QueryEvent__Done(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_QueryEvent__Debug(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_QueryEvent__MakeExternal(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 4:
		if val, err := load_QueryEvent__ExternalCall(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 5:
		if val, err := load_QueryEvent__ExternalIsa(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 6:
		if val, err := load_QueryEvent__ExternalIsSubSpecializer(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 7:
		if val, err := load_QueryEvent__ExternalIsSubclass(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 8:
		if val, err := load_QueryEvent__ExternalUnify(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 9:
		if val, err := load_QueryEvent__Result(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 10:
		if val, err := load_QueryEvent__ExternalOp(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 11:
		if val, err := load_QueryEvent__NextExternal(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for QueryEvent: %d", index)
	}
}

type QueryEvent__None struct {
}

func (*QueryEvent__None) isQueryEvent() {}

func (obj *QueryEvent__None) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(0)
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__None(deserializer serde.Deserializer) (QueryEvent__None, error) {
	var obj QueryEvent__None
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__Done struct {
	Result bool
}

func (*QueryEvent__Done) isQueryEvent() {}

func (obj *QueryEvent__Done) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeBool(obj.Result); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__Done(deserializer serde.Deserializer) (QueryEvent__Done, error) {
	var obj QueryEvent__Done
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeBool(); err == nil {
		obj.Result = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__Debug struct {
	Message string
}

func (*QueryEvent__Debug) isQueryEvent() {}

func (obj *QueryEvent__Debug) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(2)
	if err := serializer.SerializeStr(obj.Message); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__Debug(deserializer serde.Deserializer) (QueryEvent__Debug, error) {
	var obj QueryEvent__Debug
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Message = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__MakeExternal struct {
	InstanceId  uint64
	Constructor Value
}

func (*QueryEvent__MakeExternal) isQueryEvent() {}

func (obj *QueryEvent__MakeExternal) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(3)
	if err := serializer.SerializeU64(obj.InstanceId); err != nil {
		return err
	}
	if err := obj.Constructor.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__MakeExternal(deserializer serde.Deserializer) (QueryEvent__MakeExternal, error) {
	var obj QueryEvent__MakeExternal
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.InstanceId = val
	} else {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Constructor = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__ExternalCall struct {
	CallId    uint64
	Instance  Value
	Attribute string
	Args      *[]Value
	Kwargs    *map[string]Value
}

func (*QueryEvent__ExternalCall) isQueryEvent() {}

func (obj *QueryEvent__ExternalCall) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(4)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := obj.Instance.Serialize(serializer); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.Attribute); err != nil {
		return err
	}
	if err := serialize_option_vector_Value(obj.Args, serializer); err != nil {
		return err
	}
	if err := serialize_option_map_str_to_Value(obj.Kwargs, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__ExternalCall(deserializer serde.Deserializer) (QueryEvent__ExternalCall, error) {
	var obj QueryEvent__ExternalCall
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Instance = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Attribute = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_vector_Value(deserializer); err == nil {
		obj.Args = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_map_str_to_Value(deserializer); err == nil {
		obj.Kwargs = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__ExternalIsa struct {
	CallId   uint64
	Instance Value
	ClassTag string
}

func (*QueryEvent__ExternalIsa) isQueryEvent() {}

func (obj *QueryEvent__ExternalIsa) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(5)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := obj.Instance.Serialize(serializer); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.ClassTag); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__ExternalIsa(deserializer serde.Deserializer) (QueryEvent__ExternalIsa, error) {
	var obj QueryEvent__ExternalIsa
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Instance = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.ClassTag = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__ExternalIsSubSpecializer struct {
	CallId        uint64
	InstanceId    uint64
	LeftClassTag  string
	RightClassTag string
}

func (*QueryEvent__ExternalIsSubSpecializer) isQueryEvent() {}

func (obj *QueryEvent__ExternalIsSubSpecializer) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(6)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := serializer.SerializeU64(obj.InstanceId); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.LeftClassTag); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.RightClassTag); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__ExternalIsSubSpecializer(deserializer serde.Deserializer) (QueryEvent__ExternalIsSubSpecializer, error) {
	var obj QueryEvent__ExternalIsSubSpecializer
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.InstanceId = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.LeftClassTag = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.RightClassTag = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__ExternalIsSubclass struct {
	CallId        uint64
	LeftClassTag  string
	RightClassTag string
}

func (*QueryEvent__ExternalIsSubclass) isQueryEvent() {}

func (obj *QueryEvent__ExternalIsSubclass) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(7)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.LeftClassTag); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.RightClassTag); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__ExternalIsSubclass(deserializer serde.Deserializer) (QueryEvent__ExternalIsSubclass, error) {
	var obj QueryEvent__ExternalIsSubclass
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.LeftClassTag = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.RightClassTag = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__ExternalUnify struct {
	CallId          uint64
	LeftInstanceId  uint64
	RightInstanceId uint64
}

func (*QueryEvent__ExternalUnify) isQueryEvent() {}

func (obj *QueryEvent__ExternalUnify) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(8)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := serializer.SerializeU64(obj.LeftInstanceId); err != nil {
		return err
	}
	if err := serializer.SerializeU64(obj.RightInstanceId); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__ExternalUnify(deserializer serde.Deserializer) (QueryEvent__ExternalUnify, error) {
	var obj QueryEvent__ExternalUnify
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.LeftInstanceId = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.RightInstanceId = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__Result struct {
	Bindings map[string]Value
	Trace    *TraceResult
}

func (*QueryEvent__Result) isQueryEvent() {}

func (obj *QueryEvent__Result) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(9)
	if err := serialize_map_str_to_Value(obj.Bindings, serializer); err != nil {
		return err
	}
	if err := serialize_option_TraceResult(obj.Trace, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__Result(deserializer serde.Deserializer) (QueryEvent__Result, error) {
	var obj QueryEvent__Result
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserialize_map_str_to_Value(deserializer); err == nil {
		obj.Bindings = val
	} else {
		return obj, err
	}
	if val, err := deserialize_option_TraceResult(deserializer); err == nil {
		obj.Trace = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__ExternalOp struct {
	CallId   uint64
	Operator Operator
	Args     []Value
}

func (*QueryEvent__ExternalOp) isQueryEvent() {}

func (obj *QueryEvent__ExternalOp) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(10)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := obj.Operator.Serialize(serializer); err != nil {
		return err
	}
	if err := serialize_vector_Value(obj.Args, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__ExternalOp(deserializer serde.Deserializer) (QueryEvent__ExternalOp, error) {
	var obj QueryEvent__ExternalOp
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := DeserializeOperator(deserializer); err == nil {
		obj.Operator = val
	} else {
		return obj, err
	}
	if val, err := deserialize_vector_Value(deserializer); err == nil {
		obj.Args = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type QueryEvent__NextExternal struct {
	CallId   uint64
	Iterable Value
}

func (*QueryEvent__NextExternal) isQueryEvent() {}

func (obj *QueryEvent__NextExternal) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(11)
	if err := serializer.SerializeU64(obj.CallId); err != nil {
		return err
	}
	if err := obj.Iterable.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_QueryEvent__NextExternal(deserializer serde.Deserializer) (QueryEvent__NextExternal, error) {
	var obj QueryEvent__NextExternal
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeU64(); err == nil {
		obj.CallId = val
	} else {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Iterable = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Rule struct {
	Name   string
	Params []Parameter
	Body   Value
}

func (obj *Rule) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.Name); err != nil {
		return err
	}
	if err := serialize_vector_Parameter(obj.Params, serializer); err != nil {
		return err
	}
	if err := obj.Body.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeRule(deserializer serde.Deserializer) (Rule, error) {
	var obj Rule
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Name = val
	} else {
		return obj, err
	}
	if val, err := deserialize_vector_Parameter(deserializer); err == nil {
		obj.Params = val
	} else {
		return obj, err
	}
	if val, err := DeserializeValue(deserializer); err == nil {
		obj.Body = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Trace struct {
	Node     Node
	Children []Trace
}

func (obj *Trace) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := obj.Node.Serialize(serializer); err != nil {
		return err
	}
	if err := serialize_vector_Trace(obj.Children, serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeTrace(deserializer serde.Deserializer) (Trace, error) {
	var obj Trace
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeNode(deserializer); err == nil {
		obj.Node = val
	} else {
		return obj, err
	}
	if val, err := deserialize_vector_Trace(deserializer); err == nil {
		obj.Children = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type TraceResult struct {
	Trace     Trace
	Formatted string
}

func (obj *TraceResult) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	if err := obj.Trace.Serialize(serializer); err != nil {
		return err
	}
	if err := serializer.SerializeStr(obj.Formatted); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func DeserializeTraceResult(deserializer serde.Deserializer) (TraceResult, error) {
	var obj TraceResult
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeTrace(deserializer); err == nil {
		obj.Trace = val
	} else {
		return obj, err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj.Formatted = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value interface {
	isValue()
	Serialize(serializer serde.Serializer) error
}

func DeserializeValue(deserializer serde.Deserializer) (Value, error) {
	index, err := deserializer.DeserializeVariantIndex()
	if err != nil {
		return nil, err
	}

	switch index {
	case 0:
		if val, err := load_Value__Number(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 1:
		if val, err := load_Value__String(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 2:
		if val, err := load_Value__Boolean(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 3:
		if val, err := load_Value__ExternalInstance(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 4:
		if val, err := load_Value__InstanceLiteral(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 5:
		if val, err := load_Value__Dictionary(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 6:
		if val, err := load_Value__Pattern(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 7:
		if val, err := load_Value__Call(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 8:
		if val, err := load_Value__List(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 9:
		if val, err := load_Value__Variable(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 10:
		if val, err := load_Value__RestVariable(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 11:
		if val, err := load_Value__Expression(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	case 12:
		if val, err := load_Value__Partial(deserializer); err == nil {
			return &val, nil
		} else {
			return nil, err
		}

	default:
		return nil, fmt.Errorf("Unknown variant index for Value: %d", index)
	}
}

type Value__Number struct {
	Value Numeric
}

func (*Value__Number) isValue() {}

func (obj *Value__Number) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(0)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Number(deserializer serde.Deserializer) (Value__Number, error) {
	var obj Value__Number
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeNumeric(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__String string

func (*Value__String) isValue() {}

func (obj *Value__String) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(1)
	if err := serializer.SerializeStr(((string)(*obj))); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__String(deserializer serde.Deserializer) (Value__String, error) {
	var obj string
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Value__String)(obj), err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj = val
	} else {
		return ((Value__String)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Value__String)(obj), nil
}

type Value__Boolean bool

func (*Value__Boolean) isValue() {}

func (obj *Value__Boolean) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(2)
	if err := serializer.SerializeBool(((bool)(*obj))); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Boolean(deserializer serde.Deserializer) (Value__Boolean, error) {
	var obj bool
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Value__Boolean)(obj), err
	}
	if val, err := deserializer.DeserializeBool(); err == nil {
		obj = val
	} else {
		return ((Value__Boolean)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Value__Boolean)(obj), nil
}

type Value__ExternalInstance struct {
	Value ExternalInstance
}

func (*Value__ExternalInstance) isValue() {}

func (obj *Value__ExternalInstance) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(3)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__ExternalInstance(deserializer serde.Deserializer) (Value__ExternalInstance, error) {
	var obj Value__ExternalInstance
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeExternalInstance(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__InstanceLiteral struct {
	Value InstanceLiteral
}

func (*Value__InstanceLiteral) isValue() {}

func (obj *Value__InstanceLiteral) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(4)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__InstanceLiteral(deserializer serde.Deserializer) (Value__InstanceLiteral, error) {
	var obj Value__InstanceLiteral
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeInstanceLiteral(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__Dictionary struct {
	Value Dictionary
}

func (*Value__Dictionary) isValue() {}

func (obj *Value__Dictionary) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(5)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Dictionary(deserializer serde.Deserializer) (Value__Dictionary, error) {
	var obj Value__Dictionary
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeDictionary(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__Pattern struct {
	Value Pattern
}

func (*Value__Pattern) isValue() {}

func (obj *Value__Pattern) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(6)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Pattern(deserializer serde.Deserializer) (Value__Pattern, error) {
	var obj Value__Pattern
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializePattern(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__Call struct {
	Value Call
}

func (*Value__Call) isValue() {}

func (obj *Value__Call) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(7)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Call(deserializer serde.Deserializer) (Value__Call, error) {
	var obj Value__Call
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeCall(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__List []Value

func (*Value__List) isValue() {}

func (obj *Value__List) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(8)
	if err := serialize_vector_Value((([]Value)(*obj)), serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__List(deserializer serde.Deserializer) (Value__List, error) {
	var obj []Value
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Value__List)(obj), err
	}
	if val, err := deserialize_vector_Value(deserializer); err == nil {
		obj = val
	} else {
		return ((Value__List)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Value__List)(obj), nil
}

type Value__Variable string

func (*Value__Variable) isValue() {}

func (obj *Value__Variable) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(9)
	if err := serializer.SerializeStr(((string)(*obj))); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Variable(deserializer serde.Deserializer) (Value__Variable, error) {
	var obj string
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Value__Variable)(obj), err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj = val
	} else {
		return ((Value__Variable)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Value__Variable)(obj), nil
}

type Value__RestVariable string

func (*Value__RestVariable) isValue() {}

func (obj *Value__RestVariable) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(10)
	if err := serializer.SerializeStr(((string)(*obj))); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__RestVariable(deserializer serde.Deserializer) (Value__RestVariable, error) {
	var obj string
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return (Value__RestVariable)(obj), err
	}
	if val, err := deserializer.DeserializeStr(); err == nil {
		obj = val
	} else {
		return ((Value__RestVariable)(obj)), err
	}
	deserializer.DecreaseContainerDepth()
	return (Value__RestVariable)(obj), nil
}

type Value__Expression struct {
	Value Operation
}

func (*Value__Expression) isValue() {}

func (obj *Value__Expression) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(11)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Expression(deserializer serde.Deserializer) (Value__Expression, error) {
	var obj Value__Expression
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializeOperation(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}

type Value__Partial struct {
	Value Partial
}

func (*Value__Partial) isValue() {}

func (obj *Value__Partial) Serialize(serializer serde.Serializer) error {
	if err := serializer.IncreaseContainerDepth(); err != nil {
		return err
	}
	serializer.SerializeVariantIndex(12)
	if err := obj.Value.Serialize(serializer); err != nil {
		return err
	}
	serializer.DecreaseContainerDepth()
	return nil
}

func load_Value__Partial(deserializer serde.Deserializer) (Value__Partial, error) {
	var obj Value__Partial
	if err := deserializer.IncreaseContainerDepth(); err != nil {
		return obj, err
	}
	if val, err := DeserializePartial(deserializer); err == nil {
		obj.Value = val
	} else {
		return obj, err
	}
	deserializer.DecreaseContainerDepth()
	return obj, nil
}
func serialize_map_str_to_Value(value map[string]Value, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil {
		return err
	}
	offsets := make([]uint64, len(value))
	count := 0
	for k, v := range value {
		offsets[count] = serializer.GetBufferOffset()
		count += 1
		if err := serializer.SerializeStr(k); err != nil {
			return err
		}
		if err := v.Serialize(serializer); err != nil {
			return err
		}
	}
	serializer.SortMapEntries(offsets)
	return nil
}

func deserialize_map_str_to_Value(deserializer serde.Deserializer) (map[string]Value, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil {
		return nil, err
	}
	obj := make(map[string]Value)
	previous_slice := serde.Slice{0, 0}
	for i := 0; i < int(length); i++ {
		var slice serde.Slice
		slice.Start = deserializer.GetBufferOffset()
		var key string
		if val, err := deserializer.DeserializeStr(); err == nil {
			key = val
		} else {
			return nil, err
		}
		slice.End = deserializer.GetBufferOffset()
		if i > 0 {
			err := deserializer.CheckThatKeySlicesAreIncreasing(previous_slice, slice)
			if err != nil {
				return nil, err
			}
		}
		previous_slice = slice
		if val, err := DeserializeValue(deserializer); err == nil {
			obj[key] = val
		} else {
			return nil, err
		}
	}
	return obj, nil
}

func serialize_option_TraceResult(value *TraceResult, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil {
			return err
		}
		if err := (*value).Serialize(serializer); err != nil {
			return err
		}
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_option_TraceResult(deserializer serde.Deserializer) (*TraceResult, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil {
		return nil, err
	}
	if tag {
		value := new(TraceResult)
		if val, err := DeserializeTraceResult(deserializer); err == nil {
			*value = val
		} else {
			return nil, err
		}
		return value, nil
	} else {
		return nil, nil
	}
}

func serialize_option_Value(value *Value, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil {
			return err
		}
		if err := (*value).Serialize(serializer); err != nil {
			return err
		}
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_option_Value(deserializer serde.Deserializer) (*Value, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil {
		return nil, err
	}
	if tag {
		value := new(Value)
		if val, err := DeserializeValue(deserializer); err == nil {
			*value = val
		} else {
			return nil, err
		}
		return value, nil
	} else {
		return nil, nil
	}
}

func serialize_option_map_str_to_Value(value *map[string]Value, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil {
			return err
		}
		if err := serialize_map_str_to_Value((*value), serializer); err != nil {
			return err
		}
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_option_map_str_to_Value(deserializer serde.Deserializer) (*map[string]Value, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil {
		return nil, err
	}
	if tag {
		value := new(map[string]Value)
		if val, err := deserialize_map_str_to_Value(deserializer); err == nil {
			*value = val
		} else {
			return nil, err
		}
		return value, nil
	} else {
		return nil, nil
	}
}

func serialize_option_str(value *string, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil {
			return err
		}
		if err := serializer.SerializeStr((*value)); err != nil {
			return err
		}
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_option_str(deserializer serde.Deserializer) (*string, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil {
		return nil, err
	}
	if tag {
		value := new(string)
		if val, err := deserializer.DeserializeStr(); err == nil {
			*value = val
		} else {
			return nil, err
		}
		return value, nil
	} else {
		return nil, nil
	}
}

func serialize_option_vector_Value(value *[]Value, serializer serde.Serializer) error {
	if value != nil {
		if err := serializer.SerializeOptionTag(true); err != nil {
			return err
		}
		if err := serialize_vector_Value((*value), serializer); err != nil {
			return err
		}
	} else {
		if err := serializer.SerializeOptionTag(false); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_option_vector_Value(deserializer serde.Deserializer) (*[]Value, error) {
	tag, err := deserializer.DeserializeOptionTag()
	if err != nil {
		return nil, err
	}
	if tag {
		value := new([]Value)
		if val, err := deserialize_vector_Value(deserializer); err == nil {
			*value = val
		} else {
			return nil, err
		}
		return value, nil
	} else {
		return nil, nil
	}
}

func serialize_vector_Operation(value []Operation, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil {
		return err
	}
	for _, item := range value {
		if err := item.Serialize(serializer); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_vector_Operation(deserializer serde.Deserializer) ([]Operation, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil {
		return nil, err
	}
	obj := make([]Operation, length)
	for i := range obj {
		if val, err := DeserializeOperation(deserializer); err == nil {
			obj[i] = val
		} else {
			return nil, err
		}
	}
	return obj, nil
}

func serialize_vector_Parameter(value []Parameter, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil {
		return err
	}
	for _, item := range value {
		if err := item.Serialize(serializer); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_vector_Parameter(deserializer serde.Deserializer) ([]Parameter, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil {
		return nil, err
	}
	obj := make([]Parameter, length)
	for i := range obj {
		if val, err := DeserializeParameter(deserializer); err == nil {
			obj[i] = val
		} else {
			return nil, err
		}
	}
	return obj, nil
}

func serialize_vector_Trace(value []Trace, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil {
		return err
	}
	for _, item := range value {
		if err := item.Serialize(serializer); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_vector_Trace(deserializer serde.Deserializer) ([]Trace, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil {
		return nil, err
	}
	obj := make([]Trace, length)
	for i := range obj {
		if val, err := DeserializeTrace(deserializer); err == nil {
			obj[i] = val
		} else {
			return nil, err
		}
	}
	return obj, nil
}

func serialize_vector_Value(value []Value, serializer serde.Serializer) error {
	if err := serializer.SerializeLen(uint64(len(value))); err != nil {
		return err
	}
	for _, item := range value {
		if err := item.Serialize(serializer); err != nil {
			return err
		}
	}
	return nil
}

func deserialize_vector_Value(deserializer serde.Deserializer) ([]Value, error) {
	length, err := deserializer.DeserializeLen()
	if err != nil {
		return nil, err
	}
	obj := make([]Value, length)
	for i := range obj {
		if val, err := DeserializeValue(deserializer); err == nil {
			obj[i] = val
		} else {
			return nil, err
		}
	}
	return obj, nil
}
