// """Translate between Polar and the host language (Python)."""

package oso

import (
	"fmt"
	"reflect"
)

type Host struct {
	ffiPolar  PolarFfi
	classes   map[string]reflect.Type
	instances map[int]interface{}
}

func NewHost(polar PolarFfi) Host {
	classes := make(map[string]reflect.Type)
	for k, v := range CLASSES {
		classes[k] = v
	}
	instances := make(map[int]interface{})
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
	instances := make(map[int]interface{})
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

func (h Host) cacheClass(cls reflect.Type, name *string) error {
	var className string
	if name == nil {
		className = cls.Name()
	} else {
		className = *name
	}
	if v, ok := h.classes[className]; ok {
		return &DuplicateClassAliasError{name: className, cls: cls, existing: v}
	}
	h.classes[className] = cls
	return nil
}

func (h Host) getInstance(id int) (interface{}, error) {
	if v, ok := h.instances[id]; ok {
		return v, nil
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
	h.instances[instanceID] = instance
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
	leftType := reflect.TypeOf(left)
	rightType := reflect.TypeOf(right)
	if leftType == rightType && leftType.Comparable() {
		return left == right, nil
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
	return reflect.TypeOf(instance) == *class, nil
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

	return (*left).Implements(*right), nil
}

func (h Host) isSubspecializer(instanceID int, leftTag string, rightTag string) (bool, error) {
	// TODO
	return false, nil
}

//     def operator(self, op, args):
//         try:
//             if op == "Lt":
//                 return args[0] < args[1]
//             elif op == "Gt":
//                 return args[0] > args[1]
//             elif op == "Eq":
//                 return args[0] == args[1]
//             elif op == "Leq":
//                 return args[0] <= args[1]
//             elif op == "Geq":
//                 return args[0] >= args[1]
//             elif op == "Neq":
//                 return args[0] != args[1]
//             else:
//                 raise PolarRuntimeError(
//                     f"Unsupported external operation '{type(args[0])} {op} {type(args[1])}'"
//                 )
//         except TypeError:
//             raise PolarRuntimeError(
//                 f"External operation '{type(args[0])} {op} {type(args[1])}' failed."
//             )

func (h Host) toPolar(v interface{}) (*Value, error) {
	// handle nil first
	if v == nil {
		instanceID, err := h.cacheInstance(nil, nil)
		if err != nil {
			return nil, err
		}
		repr := "nil"
		inner := ValueExternalInstance{
			InstanceId:  uint64(*instanceID),
			Constructor: nil,
			Repr:        &repr,
		}
		return &Value{&inner}, nil
	}
	// check basic primitive types
	switch v.(type) {
	case bool:
		inner := ValueBoolean(v.(bool))
		return &Value{&inner}, nil
	case int, uint:
		intVal := NumericInteger(v.(int))
		inner := ValueNumber{&intVal}
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
		return h.toPolar(rtDeref.Interface())
	}

	switch rt.Kind() {
	case reflect.Slice, reflect.Array:
		vList := v.([]interface{})
		slice := make([]Value, len(vList))
		for idx, v := range vList {
			converted, err := h.toPolar(v)
			if err != nil {
				return nil, err
			}
			slice[idx] = *converted
		}
		inner := ValueList(slice)
		return &Value{&inner}, nil
	case reflect.Map:
		vMap := v.(map[string]interface{})
		fields := make(map[string]Value)
		for k, v := range vMap {
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
			return int(*number), nil
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
		return instance, nil
	}
	return nil, fmt.Errorf("Unexpected Polar type %v", v)
}
