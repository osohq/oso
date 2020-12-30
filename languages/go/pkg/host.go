// """Translate between Polar and the host language (Python)."""

package oso

import (
	"fmt"
	"reflect"
)

type Host struct {
	ffiPolar  PolarFfi
	classes   map[string]reflect.Type
	instances map[int]reflect.Value
}

func NewHost(polar PolarFfi) Host {
	classes := make(map[string]reflect.Type)
	for k, v := range CLASSES {
		classes[k] = v
	}
	instances := make(map[int]reflect.Value)
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
	instances := make(map[int]reflect.Value)
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

func (h Host) getInstance(id int) (*reflect.Value, error) {
	if v, ok := h.instances[id]; ok {
		return &v, nil
	}
	return nil, &UnregisteredInstanceError{id: id}
}

func (h Host) cacheInstance(instance interface{}, id *int) (*int, error) {
	var instanceID int
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
func (h Host) makeInstance(name string, args []interface{}, kwargs map[string]interface{}, id int) (*int, error) {
	if _, ok := h.instances[id]; ok {
		return nil, &DuplicateInstanceRegistrationError{id: id}
	}
	class, err := h.getClass(name)
	if err != nil {
		return nil, err
	}
	instance, err := InstantiateClass(*class, args, kwargs)
	if err != nil {
		return nil, err
	}
	return h.cacheInstance(*instance, &id)
}

func (h Host) unify(leftID int, rightID int) (bool, error) {
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

func (h Host) isa(value Value, classTag string) (bool, error) {
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

	// TODO: This seems like it would work for interfaces?
	return (*left).Implements(*right), nil
}

func (h Host) isSubspecializer(instanceID int, leftTag string, rightTag string) (bool, error) {
	// TODO: Not sure I can actually use these?
	// instance, err := h.getInstance(instanceID)
	// if err != nil {
	// 	return false, err
	// }
	// instanceValue := reflect.ValueOf(instance)
	leftClass, err := h.getClass(leftTag)
	if err != nil {
		return false, err
	}
	rightClass, err := h.getClass(rightTag)
	if err != nil {
		return false, err
	}
	// TODO: actually work this out
	// Idea is that if the right class is less specific
	// then it can be assigned to the left class?
	if (*rightClass).AssignableTo(*leftClass) {
		return true, nil
	}
	return false, nil
}

func (h Host) toPolar(v interface{}) (*Value, error) {
	switch v.(type) {
	case bool:
		inner := ValueBoolean(v.(bool))
		return &Value{&inner}, nil
	case int, int8, int16, int32, int64:
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
		}
		numInt := NumericInteger(intVal)
		inner := ValueNumber{&numInt}
		return &Value{&inner}, nil
	case uint, uint8, uint16, uint32, uint64:
		var uintVal int64
		switch vv := v.(type) {
		case uint:
			uintVal = int64(vv)
		case uint8:
			uintVal = int64(vv)
		case uint16:
			uintVal = int64(vv)
		case uint32:
			uintVal = int64(vv)
		case uint64:
			uintVal = int64(vv)
		}
		numInt := NumericInteger(uintVal)
		inner := ValueNumber{&numInt}
		return &Value{&inner}, nil
	case float32, float64:
		floatVal := NumericFloat(v.(float64))
		inner := ValueNumber{&floatVal}
		return &Value{&inner}, nil
	case string:
		inner := ValueString(v.(string))
		return &Value{&inner}, nil
	}

	// check composite types
	rt := reflect.ValueOf(v)
	// deref pointer
	if rt.Kind() == reflect.Ptr {
		rtDeref := rt.Elem()
		if rt.IsNil() {
			return h.toPolar(none{})
		}
		return h.toPolar(rtDeref.Interface())
	}

	switch rt.Kind() {
	case reflect.Slice, reflect.Array:
		slice := make([]Value, rt.Len())
		for i := 0; i < rt.Len(); i++ {
			converted, err := h.toPolar(rt.Index(i).Interface())
			if err != nil {
				return nil, err
			}
			slice[i] = *converted
		}
		inner := ValueList(slice)
		return &Value{&inner}, nil
	case reflect.Map:
		fields := make(map[string]Value)
		iter := rt.MapRange()
		for iter.Next() {
			k := iter.Key().String()
			v := iter.Value().Interface()
			converted, err := h.toPolar(v)
			if err != nil {
				return nil, err
			}
			fields[k] = *converted
		}
		inner := ValueDictionary{Fields: fields}
		return &Value{&inner}, nil
	default:
		instanceID, err := h.cacheInstance(v, nil)
		if err != nil {
			return nil, err
		}
		repr := fmt.Sprintf("%v", v)
		inner := ValueExternalInstance{
			InstanceId:  uint64(*instanceID),
			Constructor: nil,
			Repr:        &repr,
		}
		return &Value{&inner}, nil
	}
}

func (h Host) listToGo(v []Value) ([]interface{}, error) {
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

func (h Host) toGo(v Value) (interface{}, error) {
	switch inner := v.ValueVariant.(type) {
	case *ValueBoolean:
		return bool(*inner), nil
	case *ValueNumber:
		switch number := inner.NumericVariant.(type) {
		case *NumericInteger:
			return int64(*number), nil
		case *NumericFloat:
			return float64(*number), nil
		}
	case *ValueString:
		return string(*inner), nil
	case *ValueList:
		return h.listToGo(*inner)
	case *ValueDictionary:
		retMap := make(map[string]interface{})
		for k, v := range inner.Fields {
			ret, err := h.toGo(v)
			if err != nil {
				return nil, err
			}
			retMap[k] = ret
		}
		return retMap, nil
	case *ValueExternalInstance:
		instance, err := h.getInstance(int(inner.InstanceId))
		if err != nil {
			return nil, err
		}
		if instance == nil || !instance.IsValid() {
			return nil, nil
		}
		return (*instance).Interface(), nil
	}
	return nil, fmt.Errorf("Unexpected Polar type %v", v)
}
