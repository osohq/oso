// """Translate between Polar and the host language (Python)."""

package host

import (
	"fmt"
	"math"
	"reflect"

	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/internal/ffi"
	"github.com/osohq/go-oso/types"
	. "github.com/osohq/go-oso/types"
)

var CLASSES = make(map[string]reflect.Type)

type None struct{}

type Host struct {
	ffiPolar         ffi.PolarFfi
	classes          map[string]reflect.Type
	constructors     map[string]reflect.Value
	instances        map[uint64]reflect.Value
	fields           map[string]map[string]interface{}
	acceptExpression bool
	adapter          Adapter
}

func NewHost(polar ffi.PolarFfi) Host {
	classes := make(map[string]reflect.Type)
	for k, v := range CLASSES {
		classes[k] = v
	}
	instances := make(map[uint64]reflect.Value)
	constructors := make(map[string]reflect.Value)
	fields := make(map[string]map[string]interface{})
	return Host{
		ffiPolar:         polar,
		classes:          classes,
		instances:        instances,
		constructors:     constructors,
		fields:           fields,
		acceptExpression: false,
		adapter:          nil,
	}
}

func (h Host) Copy() Host {
	classes := make(map[string]reflect.Type)
	for k, v := range h.classes {
		classes[k] = v
	}
	instances := make(map[uint64]reflect.Value)
	for k, v := range h.instances {
		instances[k] = v
	}
	constructors := make(map[string]reflect.Value)
	for k, v := range h.constructors {
		constructors[k] = v
	}
	fields := make(map[string]map[string]interface{})
	for k, v := range h.fields {
		fields[k] = v
	}
	return Host{
		ffiPolar:     h.ffiPolar,
		classes:      classes,
		instances:    instances,
		constructors: constructors,
		fields:       fields,
		adapter:      h.adapter,
	}
}

func (h Host) GetAdapter() *Adapter {
	return &h.adapter
}

func (h Host) GetFields() map[string]map[string]interface{} {
	return h.fields
}

func (h Host) GetClass(name string) (*reflect.Type, error) {
	if v, ok := h.classes[name]; ok {
		return &v, nil
	}
	return nil, errors.NewUnregisteredClassError(name)
}

func (h Host) GetField(cls string, field string) interface{} {
	return h.fields[cls][field]
}

func (h Host) CacheClass(cls reflect.Type, name string, constructor reflect.Value, fields map[string]interface{}) error {
	if v, ok := h.classes[name]; ok {
		return errors.NewDuplicateClassAliasError(name, cls, v)
	}
	h.classes[name] = cls
	if constructor.IsValid() {
		h.constructors[name] = constructor
	}
	h.fields[name] = fields
	return nil
}

func (h Host) RegisterMros() error {
	// Go does not support inheritance, so all MROs are empty
	var err error
	for name, _ := range h.classes {
		err = h.ffiPolar.RegisterMro(name, []uint64{})
		if err != nil {
			return err
		}
	}
	return nil
}

func (h Host) getInstance(id uint64) (*reflect.Value, error) {
	if v, ok := h.instances[id]; ok {
		return &v, nil
	}
	return nil, errors.NewUnregisteredInstanceError(id)
}

func (h Host) MakeInstance(call types.ValueCall, id uint64) error {
	// Check for duplicate instance
	if _, ok := h.instances[id]; ok {
		return errors.NewDuplicateInstanceRegistrationError(id)
	}
	name := string(call.Name)
	args := call.Args

	cls, err := h.GetClass(name)
	if err != nil {
		return &errors.ErrorWithAdditionalInfo{Inner: errors.NewInvalidConstructorError(types.Value{ValueVariant: call}), Info: err.Error()}
	}
	if constructor, ok := h.constructors[name]; ok {
		results, err := h.CallFunction(constructor, args)
		if err != nil {
			return &errors.ErrorWithAdditionalInfo{Inner: errors.NewInvalidConstructorError(types.Value{ValueVariant: call}), Info: err.Error()}
		}
		if len(results) != 1 {
			return &errors.ErrorWithAdditionalInfo{Inner: errors.NewInvalidConstructorError(types.Value{ValueVariant: call}), Info: fmt.Sprintf("Constructor must retun 1 result; returned %v", len(results))}
		}
		instance := results[0]
		if instance.Type() != *cls {
			return &errors.ErrorWithAdditionalInfo{Inner: errors.NewInvalidConstructorError(types.Value{ValueVariant: call}), Info: fmt.Sprintf("Expected constructor to return %v; returned %v", *cls, instance.Type())}
		}
		h.cacheInstance(instance.Interface(), &id)
		return nil
	} else {
		return &errors.ErrorWithAdditionalInfo{Inner: errors.NewInvalidConstructorError(types.Value{ValueVariant: call}), Info: fmt.Sprintf("Missing constructor for class %v", name)}
	}
}

func (h Host) CallFunction(fn reflect.Value, termArgs []types.Term) ([]reflect.Value, error) {
	if fn.Kind() != reflect.Func {
		panic(fmt.Errorf("CallFunction expects a reflect.Func value; got: %v", fn.Kind()))
	}
	args, err := h.ListToGo(termArgs)
	if err != nil {
		return nil, err
	}
	numIn := fn.Type().NumIn()
	var end int
	if !fn.Type().IsVariadic() {
		if len(args) != numIn {
			return nil, fmt.Errorf("incorrect number of arguments. Expected %v, got %v", numIn, len(args))
		}
		end = numIn
	} else {
		// stop one before the end so we can make this a slice
		end = numIn - 1
	}

	callArgs := make([]reflect.Value, numIn)
	var results []reflect.Value

	// construct callArgs by converting them to typed values, then call method to get results
	for i := 0; i < end; i++ {
		arg := args[i]
		callArgs[i] = reflect.New(fn.Type().In(i)).Elem()
		err := SetFieldTo(callArgs[i], arg)
		if err != nil {
			return nil, err
		}
	}
	// Construct a slice for the last variadic arg for variadic methods
	if fn.Type().IsVariadic() {
		remainingArgs := args[end:]
		callArgs[end] = reflect.New(fn.Type().In(end)).Elem()
		err := SetFieldTo(callArgs[end], remainingArgs)
		if err != nil {
			return nil, err
		}
		results = fn.CallSlice(callArgs)
	} else {
		results = fn.Call(callArgs)
	}

	return results, nil
}

func (h Host) cacheInstance(instance interface{}, id *uint64) (*uint64, error) {
	var instanceID uint64
	if id == nil {
		var err error
		instanceID, err = h.ffiPolar.NewId()
		if err != nil {
			return nil, err
		}
	} else {
		instanceID = *id
	}
	h.instances[instanceID] = reflect.ValueOf(instance)
	return &instanceID, nil
}

func (h Host) Isa(value types.Term, classTag string) (bool, error) {
	instance, err := h.ToGo(value)
	if err != nil {
		return false, err
	}
	class, err := h.GetClass(classTag)
	if err != nil {
		return false, err
	}
	res := isInstance(instance, *class)
	return res, nil
}

func (h Host) IsSubclass(leftTag string, rightTag string) (bool, error) {
	left, err := h.GetClass(leftTag)
	if err != nil {
		return false, err
	}
	right, err := h.GetClass(rightTag)
	if err != nil {
		return false, err
	}

	return *left == *right, nil
}

func (h Host) IsSubspecializer(instanceID int, leftTag string, rightTag string) (bool, error) {
	return false, nil
}

func (h Host) ToPolar(v interface{}) (*Value, error) {
	if v == nil {
		return h.ToPolar(None{})
	}
	switch v := v.(type) {
	case bool:
		inner := ValueBoolean(v)
		return &Value{inner}, nil
	case int, int8, int16, int32, int64, uint, uint8, uint16, uint32, uint64:
		var intVal int64
		switch vv := v.(type) {
		case int:
			intVal = int64(vv)
		case int8:
			intVal = int64(vv)
		case int16:
			intVal = int64(vv)
		case int32:
			intVal = int64(vv)
		case int64:
			intVal = int64(vv)
		case uint:
			intVal = int64(vv)
		case uint8:
			intVal = int64(vv)
		case uint16:
			intVal = int64(vv)
		case uint32:
			intVal = int64(vv)
		case uint64:
			uintVal := uint64(vv)
			if uintVal > uint64(math.MaxInt64) {
				return nil, fmt.Errorf("Invalid integer %v, max %v", v, math.MaxInt64)
			}
			intVal = int64(vv)
		}
		inner := ValueNumber{types.NumericInteger(intVal)}
		return &Value{inner}, nil
	case float32, float64:
		var floatVal float64
		switch vv := v.(type) {
		case float32:
			floatVal = float64(vv)
		case float64:
			floatVal = float64(vv)
		}
		inner := ValueNumber{types.NumericInteger(floatVal)}
		return &Value{inner}, nil
	case string:
		inner := ValueString(v)
		return &Value{inner}, nil
	case Variable:
		return &Value{ValueVariable(v)}, nil
	case Expression:
		// Make a new array of values
		args := make([]types.Term, len(v.Args))
		for i, arg := range v.Args {
			// call toPolar on each element
			converted, err := h.ToPolar(arg)
			if err != nil {
				return nil, err
			}
			args[i] = Term{*converted}
		}
		inner := ValueExpression{
			Operator: v.Operator,
			Args:     args,
		}
		return &Value{inner}, nil
	case Value:
		return &v, nil
	case ValueVariant:
		// if its already a variant, return that
		return &Value{v}, nil
	}

	// check composite types
	rt := reflect.ValueOf(v)
	// deref pointer
	if rt.Kind() == reflect.Ptr || rt.Kind() == reflect.Interface {
		rtDeref := rt.Elem()
		if rt.IsNil() {
			// TODO: Is `nil` a reflect.Ptr?
			return h.ToPolar(None{})
		}
		return h.ToPolar(rtDeref.Interface())
	}

	switch rt.Kind() {
	case reflect.Slice, reflect.Array:
		// Make a new array of values
		slice := make([]types.Term, rt.Len())
		for i := 0; i < rt.Len(); i++ {
			// call toPolar on each element
			converted, err := h.ToPolar(rt.Index(i).Interface())
			if err != nil {
				return nil, err
			}
			slice[i] = types.Term{*converted}
		}
		inner := ValueList(slice)
		return &Value{inner}, nil
	case reflect.Map:
		fields := make(map[types.Symbol]types.Term)
		iter := rt.MapRange()
		for iter.Next() {
			// TODO(gj): error on maps w/o string keys since we're just gonna
			// stringify 'em here and something will blow up way later (probably in
			// query.go where we call
			// `reflect.ValueOf(instance).FieldByName(string(event.Attribute)`).
			k := iter.Key().String()
			v := iter.Value().Interface()
			converted, err := h.ToPolar(v)
			if err != nil {
				return nil, err
			}
			fields[types.Symbol(k)] = types.Term{*converted}
		}
		inner := ValueDictionary{Fields: fields}
		return &Value{inner}, nil
	default:
		instanceID, err := h.cacheInstance(v, nil)
		if err != nil {
			return nil, err
		}
		repr := fmt.Sprintf("%T%+v", v, v)
		classRepr := fmt.Sprintf("%T", v)
		inner := ValueExternalInstance{
			InstanceId:  *instanceID,
			Constructor: nil,
			Repr:        &repr,
			ClassRepr:   &classRepr,
		}
		return &Value{inner}, nil
	}
}

func (h Host) ListToGo(v []types.Term) ([]interface{}, error) {
	retList := make([]interface{}, len(v))
	for idx, v := range v {
		ret, err := h.ToGo(v)
		if err != nil {
			return nil, err
		}
		retList[idx] = ret
	}
	return retList, nil
}

func (h Host) ToGo(v types.Term) (interface{}, error) {
	switch inner := v.Value.ValueVariant.(type) {
	case ValueBoolean:
		return bool(inner), nil
	case ValueNumber:
		switch number := inner.NumericVariant.(type) {
		case NumericInteger:
			return int64(number), nil
		case NumericFloat:
			return float64(number), nil
		}
	case ValueString:
		return string(inner), nil
	case ValueList:
		return h.ListToGo(inner)
	case ValueDictionary:
		retMap := make(map[string]interface{})
		for k, v := range inner.Fields {
			ret, err := h.ToGo(v)
			if err != nil {
				return nil, err
			}
			retMap[string(k)] = ret
		}
		return retMap, nil
	case ValueExternalInstance:
		instance, err := h.getInstance(inner.InstanceId)
		if err != nil {
			return nil, err
		}
		if instance == nil || !instance.IsValid() {
			return nil, nil
		}
		return (*instance).Interface(), nil
	case ValueVariable:
		return Variable(inner), nil
	case ValueExpression:
		if !h.acceptExpression {
			return nil, &errors.UnexpectedExpressionError{}
		}

		// Make a new array of values
		args := make([]interface{}, len(inner.Args))
		for i, arg := range inner.Args {
			// call ToGo on each element
			converted, err := h.ToGo(arg)
			if err != nil {
				return nil, err
			}
			args[i] = converted
		}
		converted := Expression{
			Operator: inner.Operator,
			Args:     args,
		}
		return converted, nil
	case ValuePattern:
		return inner, nil
	}
	return nil, fmt.Errorf("Unexpected Polar type %v", v)
}

func (h *Host) SetAcceptExpression(acceptExpression bool) {
	h.acceptExpression = acceptExpression
}

func (h *Host) GetRelation(instance interface{}, attr string) (*Relation, error) {
	nom, err := h.getTypeName(reflect.TypeOf(instance))
	if err != nil {
		return nil, nil
	}
	switch rel := h.fields[nom][attr].(type) {
	case Relation:
		return &rel, nil
	default:
		return nil, nil
	}
}

func (h *Host) getTypeName(t reflect.Type) (string, error) {
	for nom, typ := range h.classes {
		if typ == t {
			return nom, nil
		}
	}

	return "", fmt.Errorf("Unregistered type: %v", t)
}

func (h *Host) GetRelationFields(rel FilterRelation) (string, string, error) {
	switch rec := h.fields[rel.FromTypeName][rel.FromFieldName].(type) {
	case types.Relation:
		return rec.MyField, rec.OtherField, nil
	}
	return "", "", errors.NewMissingAttributeError(h.classes[rel.FromTypeName], rel.FromFieldName)
}

// sorry bout the type
func (h *Host) SerializeTypes() (map[string]map[string]interface{}, map[string]map[string]map[string]map[string]string, error) {
	type_map := make(map[string]map[string]map[string]map[string]string, 0)

	for typ, fields := range h.fields {
		fields_map := make(map[string]map[string]map[string]string, 0)
		for k, v := range fields {
			switch t := v.(type) {
			case string:
				// chill
				fields_map[k] = map[string]map[string]string{
					"Base": {
						"class_tag": t,
					},
				}

			case types.Relation:
				// chill
				fields_map[k] = map[string]map[string]string{
					"Relation": {
						"kind":            t.Kind,
						"other_class_tag": t.OtherType,
						"my_field":        t.MyField,
						"other_field":     t.OtherField,
					},
				}
			default:
				return nil, nil, fmt.Errorf("type must be a string typename or a Relation struct: got %v", v)
			}
		}
		type_map[typ] = fields_map
	}

	return h.fields, type_map, nil
}

func (h *Host) SetDataFilteringAdapter(adapter Adapter) {
	h.adapter = adapter
}

func (h *Host) BuildQuery(filter *Filter) (interface{}, error) {
	if h.adapter == nil {
		return nil, fmt.Errorf("must register an adapter to use data filtering")
	}

	return (h.adapter).BuildQuery(filter)
}

func (h *Host) ExecuteQuery(query interface{}) ([]interface{}, error) {
	if h.adapter == nil {
		return nil, fmt.Errorf("must register an adapter to use data filtering")
	}

	return (h.adapter).ExecuteQuery(query)
}

func (h *Host) ParseValues(filter *Filter) error {
	for i := range filter.Conditions {
		for j := range filter.Conditions[i] {
			switch t := filter.Conditions[i][j].Rhs.DatumVariant.(type) {
			case Immediate:
				go_value, err := h.ToGo(types.Term{t.Value.(Value)})
				if err != nil {
					return err
				}
				datum := Datum{Immediate{go_value}}
				filter.Conditions[i][j].Rhs = datum
			}
			switch t := filter.Conditions[i][j].Lhs.DatumVariant.(type) {
			case Immediate:
				go_value, err := h.ToGo(types.Term{t.Value.(Value)})
				if err != nil {
					return err
				}
				datum := Datum{Immediate{go_value}}
				filter.Conditions[i][j].Lhs = datum
			}
		}
	}
	return nil
}
