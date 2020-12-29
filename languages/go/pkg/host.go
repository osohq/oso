// """Translate between Polar and the host language (Python)."""

package oso

import (
	"errors"
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

func (h Host) cacheClass(cls interface{}, name *string) error {
	var className string
	if name == nil {
		className = reflect.TypeOf(className).Name()
	} else {
		className = *name
	}
	if v, ok := h.classes[className]; ok {
		return &DuplicateClassAliasError{name: className, cls: reflect.TypeOf(cls), existing: v}
	}
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
	instance := reflect.New(*class)
	for idx, arg := range args {
		f := instance.Field(idx)
		if f.IsValid() && f.CanSet() {
			f.Set(reflect.ValueOf(arg))
		} else {
			return nil, fmt.Errorf("cannot set field %v", f)
		}
	}
	for k, v := range kwargs {
		f := instance.FieldByName(k)
		if f.IsValid() && f.CanSet() {
			f.Set(reflect.ValueOf(v))
		} else {
			return nil, fmt.Errorf("cannot set field %v", f)
		}
	}
	return h.cacheInstance(instance, nil)
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
	if left.Type() == right.Type() && left.Type().Comparable() {
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
	// check basic primitive types
	switch v.(type) {
	case bool:
		inner := ValueBoolean(v.(bool))
		return &Value{&inner}, nil
	case int:
	case uint:
		intVal := NumericInteger(v.(int))
		inner := ValueNumber{&intVal}
		return &Value{&inner}, nil
	case float32:
	case float64:
		floatVal := NumericFloat(v.(float64))
		inner := ValueNumber{&floatVal}
		return &Value{&inner}, nil
	case string:
		inner := ValueString(v.(string))
		return &Value{&inner}, nil
	}

	// check composite types
	rt := reflect.TypeOf(v)
	switch rt.Kind() {
	case reflect.Slice:
	case reflect.Array:
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

	return nil, errors.New("unreachable")
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
		retList := make([]interface{}, len(*inner))
		for idx, v := range *inner {
			ret, err := h.toGo(v)
			if err != nil {
				return nil, err
			}
			retList[idx] = ret
		}
		return retList, nil
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
		return h.getInstance(int(inner.InstanceId))
	}
	return nil, fmt.Errorf("Unexpected Polar type %v", v)
}

//         tag = [*value][0]
//         if tag in ["String", "Boolean"]:
//             return value[tag]
//         elif tag == "Number":
//             number = [*value[tag].values()][0]
//             if "Float" in value[tag]:
//                 if number == "Infinity":
//                     return inf
//                 elif number == "-Infinity":
//                     return -inf
//                 elif number == "NaN":
//                     return nan
//                 else:
//                     if not isinstance(number, float):
//                         raise PolarRuntimeError(
//                             f'Expected a floating point number, got "{number}"'
//                         )
//             return number
//         elif tag == "List":
//             return [self.to_python(e) for e in value[tag]]
//         elif tag == "Dictionary":
//             return {k: self.to_python(v) for k, v in value[tag]["fields"].items()}
//         elif tag == "ExternalInstance":
//             return self.get_instance(value[tag]["instance_id"])
//         elif tag == "Call":
//             return Predicate(
//                 name=value[tag]["name"],
//                 args=[self.to_python(v) for v in value[tag]["args"]],
//             )
//         elif tag == "Variable":
//             return Variable(value[tag])
//         elif tag == "Expression":
//             args = list(map(self.to_python, value[tag]["args"]))
//             operator = value[tag]["operator"]

//             return Expression(operator, args)
//         elif tag == "Pattern":
//             pattern_tag = [*value[tag]][0]
//             if pattern_tag == "Instance":
//                 instance = value[tag]["Instance"]
//                 return Pattern(instance["tag"], instance["fields"]["fields"])
//             elif pattern_tag == "Dictionary":
//                 dictionary = value[tag]["Dictionary"]
//                 return Pattern(None, dictionary["fields"])
//             else:
//                 raise UnexpectedPolarTypeError("Pattern: " + value[tag])

//         raise UnexpectedPolarTypeError(tag)
