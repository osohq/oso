package oso

import (
	"fmt"
	"reflect"
)

func String(s string) *string {
	return &s
}

func setFieldTo(field reflect.Value, input interface{}) error {
	// todo: explain how this works and why.
	if !field.CanSet() {
		return fmt.Errorf("cannot set field")
	}
	fieldType := field.Type()
	switch fieldKind := field.Kind(); fieldKind {
	case reflect.Array, reflect.Slice:
		inputArray, ok := input.([]interface{})
		field.Set(reflect.MakeSlice(field.Type(), len(inputArray), len(inputArray)))
		if !ok {
			return fmt.Errorf("Cannot assign to array from %s", reflect.TypeOf(input).Kind())
		}
		for idx, v := range inputArray {
			err := setFieldTo(field.Index(idx), v)
			if err != nil {
				return err
			}
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
	case reflect.Bool:
		field.SetBool(reflect.ValueOf(input).Convert(fieldType).Bool())
	case reflect.Float32, reflect.Float64:
		field.SetFloat(reflect.ValueOf(input).Convert(fieldType).Float())
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		field.SetInt(reflect.ValueOf(input).Convert(fieldType).Int())
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		field.SetUint(reflect.ValueOf(input).Convert(fieldType).Uint())
	case reflect.String:
		field.SetString(reflect.ValueOf(input).Convert(fieldType).String())
	default:
		valInput := reflect.ValueOf(input)
		valid := field.IsValid()
		canSet := field.CanSet()
		inputType := valInput.Type()
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
		if !f.IsValid() {
			return fmt.Errorf("Cannot set field #%v", idx)
		}
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
		if !f.IsValid() {
			return fmt.Errorf("Cannot set field %v", k)
		}
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
	case reflect.Array, reflect.Slice:
		if len(kwargs) != 0 {
			return nil, fmt.Errorf("Cannot assign kwargs to a class of type: %s", class.Kind())
		}
		err := setFieldTo(instance, args)
		if err != nil {
			return nil, err
		}
	case reflect.Map:
		if len(args) != 0 {
			return nil, fmt.Errorf("Cannot assign args to a class of type: %s", class.Kind())
		}
		err := setFieldTo(instance, kwargs)
		if err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("Cannot instantiate a class of type: %s", class.Kind())
	}
	instanceInterface := instance.Interface()
	return &instanceInterface, nil
}
