package oso

import (
	"fmt"
	"reflect"
)

func setFieldTo(field reflect.Value, input interface{}) error {
	switch field.Kind() {
	case reflect.Array, reflect.Slice:
		inputArray := input.([]interface{})
		for _, v := range inputArray {
			elem := reflect.New(field.Type().Elem())
			err := setFieldTo(elem, v)
			if err != nil {
				return err
			}
			reflect.Append(field, elem)
		}
		return nil
	case reflect.Map:
		inputMap := input.(map[string]interface{})
		for k, v := range inputMap {
			entry := reflect.New(field.Type().Elem())
			err := setFieldTo(entry, v)
			if err != nil {
				return err
			}
			reflect.Append(field, entry)
			field.SetMapIndex(reflect.ValueOf(k), entry)
		}
	default:
		valInput := reflect.ValueOf(input)
		if field.IsValid() && field.CanSet() && valInput.Type().ConvertibleTo(field.Type()) {
			field.Set(valInput.Convert(field.Type()))
		} else {
			return fmt.Errorf("cannot set field %v\nIsValid: %v\nCanSet: %v\n%v -> %v: %v", field, field.IsValid(), field.CanSet(), valInput.Type(), field.Type(), valInput.Type().ConvertibleTo(field.Type()))
		}
	}
	return nil
}

// InstantiateClass sets the fields of a new instance of `class` to those provided in `args` and `kwargs`
func InstantiateClass(class reflect.Type, args []interface{}, kwargs map[string]interface{}) (*interface{}, error) {
	instancePtr := reflect.New(class)
	instance := instancePtr.Elem()
	for idx, arg := range args {
		f := instance.Field(idx)
		err := setFieldTo(f, arg)
		if err != nil {
			return nil, err
		}
	}
	for k, v := range kwargs {
		f := instance.FieldByName(k)
		err := setFieldTo(f, v)
		if err != nil {
			return nil, err
		}
	}
	instanceInterface := instance.Interface()
	return &instanceInterface, nil
}
