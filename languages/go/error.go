package oso

import (
	"fmt"
	"reflect"
)

type DuplicateClassAliasError struct {
	name     string
	cls      reflect.Type
	existing reflect.Type
}

func NewDuplicateClassAliasError(name string, cls reflect.Type, existing reflect.Type) error {
	return &DuplicateClassAliasError{name: name, cls: cls, existing: existing}
}

func (e *DuplicateClassAliasError) Error() string {
	return fmt.Sprintf("Attempted to alias %v as '%s', but %v already has that alias.", e.cls, e.name, e.existing)
}

type DuplicateInstanceRegistrationError struct {
	id uint64
}

func (e *DuplicateInstanceRegistrationError) Error() string {
	return fmt.Sprintf("Attempted to register instance %d, but an instance with that ID already exists.", e.id)
}

type InlineQueryFailedError struct {
	source string
}

func (e *InlineQueryFailedError) Error() string {
	return fmt.Sprintf("Inline query failed: %s", e.source)
}

type MissingAttributeError struct {
	instance interface{}
	field    string
}

func (e *MissingAttributeError) Error() string {
	return fmt.Sprintf("'%v' object has no attribute '%s'", e.instance, e.field)
}

type InvalidCallError struct {
	instance interface{}
	field    string
}

func (e *InvalidCallError) Error() string {
	return fmt.Sprintf("%v.%s is not a function", e.instance, e.field)
}

type InvalidIteratorError struct {
	instance Value
}

func (e *InvalidIteratorError) Error() string {
	return fmt.Sprintf("%v is not iterable", e.instance)
}

type InvalidConstructorError struct {
	ctor Value
}

func (e *InvalidConstructorError) Error() string {
	return fmt.Sprintf("%v is not a constructor", e.ctor)
}

type InvalidQueryEventError struct {
	event string
}

func (e *InvalidQueryEventError) Error() string {
	return fmt.Sprintf("Invalid query event: %s", e.event)
}

type KwargsError struct {
}

func (e *KwargsError) Error() string {
	return fmt.Sprintf("Go does not support keyword arguments")
}

type PolarFileExtensionError struct {
	file string
}

func (e *PolarFileExtensionError) Error() string {
	return fmt.Sprintf("Polar files must have .polar extension. Offending file: %s", e.file)
}

type PolarFileNotFoundError struct {
	file string
}

func (e *PolarFileNotFoundError) Error() string {
	return fmt.Sprintf("Could not find file: %s", e.file)
}

type UnimplementedOperationError struct {
	operation string
}

func (e *UnimplementedOperationError) Error() string {
	return fmt.Sprintf("%s are unimplemented in the oso Go library", e.operation)
}

type UnregisteredClassError struct {
	name string
}

func (e *UnregisteredClassError) Error() string {
	return fmt.Sprintf("Unregistered class: %s", e.name)
}

type UnregisteredInstanceError struct {
	id uint64
}

func (e *UnregisteredInstanceError) Error() string {
	return fmt.Sprintf("Unregistered instance: %d.", e.id)
}

func (e *FormattedPolarError) Error() string {
	return fmt.Sprintf("Error: %#v\n%s", e.Kind.ErrorKindVariant, e.Formatted)
}

type ErrorWithAdditionalInfo struct {
	Inner error
	Info  string
}

func (e *ErrorWithAdditionalInfo) Error() string {
	return fmt.Sprintf("%s\n%s", e.Info, e.Inner)
}
