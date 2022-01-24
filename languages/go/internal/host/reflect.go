package host

import (
	"fmt"
	"reflect"
)

// Checking for type equality is currently done with exact matching
// We will not check if Go would permit a conversion between types or whether a
// NewType "matches" its wrapped inner type. This is because doing so has the
// side-effect of allowing runtime type confusion between structs of identical
// schemas at query time.
func isInstance(instance interface{}, class reflect.Type) bool {
	instanceType := reflect.TypeOf(instance)
	res := instanceType == class
	return res
}

func String(s string) *string {
	return &s
}

func SetFieldTo(field reflect.Value, input interface{}) error {
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
			err := SetFieldTo(field.Index(idx), v)
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
			err := SetFieldTo(entry, v)
			if err != nil {
				return err
			}
			field.SetMapIndex(reflect.ValueOf(k), entry)
		}
	case reflect.Ptr:
		deref := field.Elem()
		return SetFieldTo(deref, input)
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
