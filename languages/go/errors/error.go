package errors

import (
	"fmt"
	"reflect"

	"github.com/osohq/go-oso/types"
)

type DuplicateClassAliasError struct {
	name     string
	cls      reflect.Type
	existing reflect.Type
}

func NewDuplicateClassAliasError(name string, cls reflect.Type, existing reflect.Type) *DuplicateClassAliasError {
	return &DuplicateClassAliasError{name: name, cls: cls, existing: existing}
}

func (e *DuplicateClassAliasError) Error() string {
	return fmt.Sprintf("Attempted to alias %v as '%s', but %v already has that alias.", e.cls, e.name, e.existing)
}

type DuplicateInstanceRegistrationError struct {
	id uint64
}

func NewDuplicateInstanceRegistrationError(id uint64) *DuplicateInstanceRegistrationError {
	return &DuplicateInstanceRegistrationError{id: id}
}

func (e *DuplicateInstanceRegistrationError) Error() string {
	return fmt.Sprintf("Attempted to register instance %d, but an instance with that ID already exists.", e.id)
}

type InlineQueryFailedError struct {
	source string
}

func NewInlineQueryFailedError(source string) *InlineQueryFailedError {
	return &InlineQueryFailedError{source: source}
}

func (e *InlineQueryFailedError) Error() string {
	return fmt.Sprintf("Inline query failed: %s", e.source)
}

type MissingAttributeError struct {
	instance interface{}
	field    string
}

func NewMissingAttributeError(instance interface{}, field string) *MissingAttributeError {
	return &MissingAttributeError{instance: instance, field: field}
}

func (e *MissingAttributeError) Error() string {
	return fmt.Sprintf("'%v' object has no attribute '%s'", e.instance, e.field)
}

type InvalidCallError struct {
	instance interface{}
	field    string
}

func NewInvalidCallError(instance interface{}, field string) *InvalidCallError {
	return &InvalidCallError{instance: instance, field: field}
}

func (e *InvalidCallError) Error() string {
	return fmt.Sprintf("%v.%s is not a function", e.instance, e.field)
}

type InvalidIteratorError struct {
	instance interface{}
}

func NewInvalidIteratorError(instance interface{}) *InvalidIteratorError {
	return &InvalidIteratorError{instance: instance}
}

func (e *InvalidIteratorError) Error() string {
	return fmt.Sprintf("%v is not iterable", e.instance)
}

type InvalidConstructorError struct {
	ctor types.Value
}

func NewInvalidConstructorError(ctor types.Value) *InvalidConstructorError {
	return &InvalidConstructorError{ctor: ctor}
}

func (e *InvalidConstructorError) Error() string {
	return fmt.Sprintf("%v is not a constructor", e.ctor)
}

type InvalidQueryEventError struct {
	event string
}

func NewInvalidQueryEventError(event string) *InvalidQueryEventError {
	return &InvalidQueryEventError{event: event}
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

func NewPolarFileExtensionError(file string) *PolarFileExtensionError {
	return &PolarFileExtensionError{file: file}
}

func (e *PolarFileExtensionError) Error() string {
	return fmt.Sprintf("Polar files must have .polar extension. Offending file: %s", e.file)
}

type PolarFileNotFoundError struct {
	file string
}

func NewPolarFileNotFoundError(file string) *PolarFileNotFoundError {
	return &PolarFileNotFoundError{file: file}
}

func (e *PolarFileNotFoundError) Error() string {
	return fmt.Sprintf("Could not find file: %s", e.file)
}

type UnimplementedOperationError struct {
	operation string
}

func NewUnimplementedOperationError(operation string) *UnimplementedOperationError {
	return &UnimplementedOperationError{operation: operation}
}

func (e *UnimplementedOperationError) Error() string {
	return fmt.Sprintf("%s are unimplemented in the oso Go library", e.operation)
}

type UnregisteredClassError struct {
	name string
}

func NewUnregisteredClassError(name string) *UnregisteredClassError {
	return &UnregisteredClassError{name: name}
}

func (e *UnregisteredClassError) Error() string {
	return fmt.Sprintf("Unregistered class: %s", e.name)
}

type UnregisteredInstanceError struct {
	id uint64
}

func NewUnregisteredInstanceError(id uint64) *UnregisteredInstanceError {
	return &UnregisteredInstanceError{id: id}
}

func (e *UnregisteredInstanceError) Error() string {
	return fmt.Sprintf("Unregistered instance: %d.", e.id)
}

// FormattedPolarError struct
type FormattedPolarError struct {
	// Kind
	Kind types.ErrorKind `json:"kind"`
	// Formatted
	Formatted string `json:"formatted"`
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

type NotFoundError struct{}

func (e *NotFoundError) Error() string {
	return "Oso Not Found Error -- the current user does not have permission to " +
		"read the given resource. You should handle this error by returning a 404 " +
		"error to the client."
}

type ForbiddenError struct{}

func (e *ForbiddenError) Error() string {
	return "Oso Forbidden Error -- the requested action was not allowed for the " +
		"given resource. Most often, you should handle this error by returning a " +
		"403 error to the client."
}

type UnexpectedExpressionError struct{}

func (e *UnexpectedExpressionError) Error() string {
	return "Received Expression from Polar VM. The Expression type is only supported when " +
		"using data filtering features. Did you perform an operation over an unbound variable " +
		"in your policy?\n\n" +
		"To silence this error and receive an Expression result, call query.SetAcceptExpression(true) " +
		"on a query (e.g. created by calling Oso.NewQueryFromRule)."

}
