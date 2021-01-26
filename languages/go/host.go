// """Translate between Polar and the host language (Python)."""

package oso

import (
	"fmt"
	"reflect"
)

type Host struct {
	ffiPolar  PolarFfi
	classes   map[string]reflect.Type
	instances map[uint64]reflect.Value
}

func NewHost(polar PolarFfi) Host {
	classes := make(map[string]reflect.Type)
	for k, v := range CLASSES {
		classes[k] = v
	}
	instances := make(map[uint64]reflect.Value)
	return Host{
		ffiPolar:  polar,
		classes:   classes,
		instances: instances,
	}
}

func (h Host) copy() Host {
	classes := make(map[string]reflect.Type)
	for k, v := range h.classes {
		classes[k] = v
	}
	instances := make(map[uint64]reflect.Value)
	for k, v := range h.instances {
		instances[k] = v
	}
	return Host{
		ffiPolar:  h.ffiPolar,
		classes:   classes,
		instances: instances,
	}
}

func (h Host) getClass(name string) (*reflect.Type, error) {
	if v, ok := h.classes[name]; ok {
		return &v, nil
	}
	return nil, &UnregisteredClassError{name: name}
}

func (h Host) cacheClass(cls reflect.Type, name string) error {
	if v, ok := h.classes[name]; ok {
		return &DuplicateClassAliasError{name: name, cls: cls, existing: v}
	}
	h.classes[name] = cls
	return nil
}

func (h Host) getInstance(id uint64) (*reflect.Value, error) {
	if v, ok := h.instances[id]; ok {
		return &v, nil
	}
	return nil, &UnregisteredInstanceError{id: id}
}

func (h Host) cacheInstance(instance interface{}, id *uint64) (*uint64, error) {
	var instanceID uint64
	if id == nil {
		var err error
		instanceID, err = h.ffiPolar.newId()
		if err != nil {
			return nil, err
		}
	} else {
		instanceID = *id
	}
	h.instances[instanceID] = reflect.ValueOf(instance)
	return &instanceID, nil
}

// makeInstance construct and cache a Go instance.
// TODO: should we even allow any arguments?
func (h Host) makeInstance(name string, args []interface{}, kwargs map[string]interface{}, id uint64) (*uint64, error) {
	return nil, fmt.Errorf("Constructing new instance is not supported in Go")
	// if _, ok := h.instances[id]; ok {
	// 	return nil, &DuplicateInstanceRegistrationError{id: id}
	// }
	// class, err := h.getClass(name)
	// if err != nil {
	// 	return nil, err
	// }
	// instance, err := InstantiateClass(*class, args, kwargs)
	// if err != nil {
	// 	return nil, err
	// }
	// return h.cacheInstance(*instance, &id)
}

func (h Host) unify(leftID uint64, rightID uint64) (bool, error) {
	left, err1 := h.getInstance(leftID)
	right, err2 := h.getInstance(rightID)
	if err1 != nil {
		return false, err1
	}
	if err2 != nil {
		return false, err2
	}
	if leftEq, ok := left.Interface().(Comparer); ok {
		if rightEq, ok := right.Interface().(Comparer); ok {
			return leftEq.Equal(rightEq), nil
		}
	}
	return reflect.DeepEqual(left, right), nil
}

func (h Host) isa(value Term, classTag string) (bool, error) {
	instance, err := h.toGo(value)
	if err != nil {
		return false, err
	}
	class, err := h.getClass(classTag)
	if err != nil {
		return false, err
	}
	instanceType := reflect.TypeOf(instance)
	res := instanceType.ConvertibleTo(*class)
	return res, nil
}

func (h Host) isSubclass(leftTag string, rightTag string) (bool, error) {
	left, err := h.getClass(leftTag)
	if err != nil {
		return false, err
	}
	right, err := h.getClass(rightTag)
	if err != nil {
		return false, err
	}

	return *left == *right, nil
}

func (h Host) isSubspecializer(instanceID int, leftTag string, rightTag string) (bool, error) {
	return false, nil
}

func (h Host) toPolar(v interface{}) (*Value, error) {
	if v == nil {
		return h.toPolar(none{})
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
			intVal = int64(vv)
		}
		inner := ValueNumber{NumericInteger(intVal)}
		return &Value{inner}, nil
	case float32, float64:
		var floatVal float64
		switch vv := v.(type) {
		case float32:
			floatVal = float64(vv)
		case float64:
			floatVal = float64(vv)
		}
		inner := ValueNumber{NumericInteger(floatVal)}
		return &Value{inner}, nil
	case string:
		inner := ValueString(v)
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
	if rt.Kind() == reflect.Ptr {
		rtDeref := rt.Elem()
		if rt.IsNil() {
			// TODO: Is `nil` a reflect.Ptr?
			return h.toPolar(none{})
		}
		return h.toPolar(rtDeref.Interface())
	}

	switch rt.Kind() {
	case reflect.Slice, reflect.Array:
		// Make a new array of values
		slice := make([]Term, rt.Len())
		for i := 0; i < rt.Len(); i++ {
			// call toPolar on each element
			converted, err := h.toPolar(rt.Index(i).Interface())
			if err != nil {
				return nil, err
			}
			slice[i] = Term{*converted}
		}
		inner := ValueList(slice)
		return &Value{inner}, nil
	case reflect.Map:
		fields := make(map[Symbol]Term)
		iter := rt.MapRange()
		for iter.Next() {
			k := iter.Key().String()
			v := iter.Value().Interface()
			converted, err := h.toPolar(v)
			if err != nil {
				return nil, err
			}
			fields[Symbol(k)] = Term{*converted}
		}
		inner := ValueDictionary{Fields: fields}
		return &Value{inner}, nil
	default:
		instanceID, err := h.cacheInstance(v, nil)
		if err != nil {
			return nil, err
		}
		repr := fmt.Sprintf("%v", v)
		inner := ValueExternalInstance{
			InstanceId:  *instanceID,
			Constructor: nil,
			Repr:        &repr,
		}
		return &Value{inner}, nil
	}
}

func (h Host) listToGo(v []Term) ([]interface{}, error) {
	retList := make([]interface{}, len(v))
	for idx, v := range v {
		ret, err := h.toGo(v)
		if err != nil {
			return nil, err
		}
		retList[idx] = ret
	}
	return retList, nil
}

func (h Host) toGo(v Term) (interface{}, error) {
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
		return h.listToGo(inner)
	case ValueDictionary:
		retMap := make(map[string]interface{})
		for k, v := range inner.Fields {
			ret, err := h.toGo(v)
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
		return inner, nil
	}
	return nil, fmt.Errorf("Unexpected Polar type %v", v)
}
