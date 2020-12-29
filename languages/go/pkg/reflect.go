package oso

import (
	"fmt"
	"reflect"
)

func setFieldTo(field reflect.Value, input interface{}) error {
	switch fieldKind := field.Kind(); fieldKind {
	case reflect.Array, reflect.Slice:
		inputArray, ok := input.([]interface{})
		field.Set(reflect.MakeSlice(field.Type(), len(inputArray), len(inputArray)))
		if !ok {
			return fmt.Errorf("Cannot assign to array from %s", reflect.TypeOf(input).Kind())
		}
		for _, v := range inputArray {
			elem := reflect.New(field.Type().Elem()).Elem()
			err := setFieldTo(elem, v)
			if err != nil {
				return err
			}
			reflect.Append(field, elem)
		}
		return nil
	case reflect.Map:
		inputMap := input.(map[string]interface{})
		field.Set(reflect.MakeMap(field.Type()))
		for k, v := range inputMap {
			entry := reflect.New(field.Type().Elem())
			err := setFieldTo(entry, v)
			if err != nil {
				return err
			}
			field.SetMapIndex(reflect.ValueOf(k), entry)
		}
	case reflect.Ptr:
		deref := field.Elem()
		return setFieldTo(deref, input)
	default:
		valInput := reflect.ValueOf(input)
		valid := field.IsValid()
		canSet := field.CanSet()
		inputType := valInput.Type()
		fieldType := field.Type()
		if valid && canSet && inputType.ConvertibleTo(fieldType) {
			field.Set(valInput.Convert(fieldType))
		} else {
			return fmt.Errorf("cannot set field %s\nIsValid: %v\nCanSet: %v\n%v -> %v: %v", fieldKind, valid, canSet, inputType, fieldType, valInput.Type().ConvertibleTo(fieldType))
		}
	}
	return nil
}

func setStructFields(instance reflect.Value, args []interface{}) error {
	for idx, arg := range args {
		f := instance.Field(idx)
		err := setFieldTo(f, arg)
		if err != nil {
			return err
		}
	}
	return nil
}

func setMapFields(instance reflect.Value, kwargs map[string]interface{}) error {
	for k, v := range kwargs {
		f := instance.FieldByName(k)
		err := setFieldTo(f, v)
		if err != nil {
			return err
		}
	}
	return nil
}

// InstantiateClass sets the fields of a new instance of `class` to those provided in `args` and `kwargs`
func InstantiateClass(class reflect.Type, args []interface{}, kwargs map[string]interface{}) (*interface{}, error) {
	instancePtr := reflect.New(class)
	instance := instancePtr.Elem()

	switch class.Kind() {
	case reflect.Struct:
		err := setStructFields(instance, args)
		if err != nil {
			return nil, err
		}
		err = setMapFields(instance, kwargs)
		if err != nil {
			return nil, err
		}
		break
	case reflect.Array, reflect.Slice:
		if len(kwargs) != 0 {
			return nil, fmt.Errorf("Cannot assign kwargs to a class of type: %s", class.Kind())
		}
		err := setFieldTo(instance, args)
		if err != nil {
			return nil, err
		}
		break
	case reflect.Map:
		if len(args) != 0 {
			return nil, fmt.Errorf("Cannot assign args to a class of type: %s", class.Kind())
		}
		err := setFieldTo(instance, kwargs)
		if err != nil {
			return nil, err
		}
		break
	default:
		return nil, fmt.Errorf("Cannot instantiate a class of type: %s", class.Kind())
	}
	instanceInterface := instance.Interface()
	return &instanceInterface, nil
}
