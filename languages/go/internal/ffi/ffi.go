package ffi

// #cgo CFLAGS: -g -Wall
// #include <stdint.h>
// #include <stdlib.h>
// #include "native/polar.h"
// #cgo linux,amd64 LDFLAGS: ${SRCDIR}/native/linux/libpolar.a -ldl -lm
// #cgo darwin,amd64 LDFLAGS: ${SRCDIR}/native/macos/libpolar.a -ldl -lm
// #cgo windows,amd64 LDFLAGS: ${SRCDIR}/native/windows/libpolar.a -lm -lws2_32 -luserenv
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"

	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/types"
)

/*
Reads a c string from polar core to a go string and frees the c string.
*/
func readStr(cStr *C.char) string {
	goStr := C.GoString(cStr)
	C.string_free(cStr)
	return goStr
}

func ffiSerialize(input interface{}) (*C.char, error) {
	json, err := json.Marshal(input)
	if err != nil {
		return nil, err
	}
	return C.CString(string(json)), nil
}

type PolarFfi struct {
	ptr *C.polar_Polar
}

func NewPolarFfi() PolarFfi {
	polarPtr := C.polar_new()
	return PolarFfi{
		ptr: polarPtr,
	}
}

func (p *PolarFfi) delete() {
	C.polar_free(p.ptr)
	p = nil
}

func getError() error {
	err := C.polar_get_error()
	errStr := readStr(err)
	var polarError errors.FormattedPolarError
	jsonErr := json.Unmarshal([]byte(errStr), &polarError)
	if jsonErr != nil {
		return jsonErr
	}
	return &polarError
}

type ffiInterface interface {
	nextMessage() *C.char
}

func (p PolarFfi) nextMessage() *C.char {
	return C.polar_next_polar_message(p.ptr)
}

func processMessages(i ffiInterface) {
	for {
		msgPtr := i.nextMessage()
		if msgPtr == nil {
			return
		}
		message := readStr(msgPtr)
		var messageStruct types.Message
		err := json.Unmarshal([]byte(message), &messageStruct)

		if err != nil {
			panic(err)
		}
		switch messageStruct.Kind.MessageKindVariant.(type) {
		case types.MessageKindPrint:
			fmt.Printf("%s\n", messageStruct.Msg)
			break
		case types.MessageKindWarning:
			fmt.Printf("WARNING: %s\n", messageStruct.Msg)
			break
		default:
			fmt.Printf("Unexpected message: %#v\n", messageStruct)
		}
	}
}

func (p PolarFfi) NewId() (uint64, error) {
	id := C.polar_get_external_id(p.ptr)
	if id == 0 {
		return 0, getError()
	}
	return uint64(id), nil
}

func (p PolarFfi) Load(s string, filename *string) error {
	cString := C.CString(s)
	defer C.free(unsafe.Pointer(cString))
	var cFilename *C.char
	if filename != nil {
		cFilename = C.CString(*filename)
		defer C.free(unsafe.Pointer(cFilename))
	}
	result := C.polar_load(p.ptr, cString, cFilename)
	processMessages(p)
	if result == 0 {
		return getError()
	}
	return nil
}

func (p PolarFfi) ClearRules() error {
	result := C.polar_clear_rules(p.ptr)
	processMessages(p)
	if result == 0 {
		return getError()
	}
	return nil
}

func (p PolarFfi) NewQueryFromStr(queryStr string) (*QueryFfi, error) {
	cs := C.CString(queryStr)
	defer C.free(unsafe.Pointer(cs))
	result := C.polar_new_query(p.ptr, cs, 0)
	processMessages(p)
	if result == nil {
		return nil, getError()
	}
	return newQueryFfi(result), nil
}

func (p PolarFfi) NewQueryFromTerm(queryTerm types.Term) (*QueryFfi, error) {
	json, err := ffiSerialize(queryTerm)
	defer C.free(unsafe.Pointer(json))
	if err != nil {
		return nil, err
	}
	result := C.polar_new_query_from_term(p.ptr, json, 0)
	processMessages(p)
	if result == nil {
		return nil, getError()
	}
	return newQueryFfi(result), nil
}

func (p PolarFfi) NextInlineQuery() (*QueryFfi, error) {
	queryPtr := C.polar_next_inline_query(p.ptr, 0)
	processMessages(p)
	if queryPtr == nil {
		// TODO: we don't have any way of signaling this failing?
		return nil, nil
	}
	return newQueryFfi(queryPtr), nil
}

func (p PolarFfi) RegisterConstant(term types.Term, name string) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cTerm, err := ffiSerialize(term)
	defer C.free(unsafe.Pointer(cTerm))
	if err != nil {
		return err
	}
	result := C.polar_register_constant(p.ptr, cName, cTerm)
	processMessages(p)
	if result == 0 {
		return getError()
	}
	return nil
}

type QueryFfi struct {
	ptr *C.polar_Query
}

func newQueryFfi(ptr *C.polar_Query) *QueryFfi {
	return &QueryFfi{
		ptr: ptr,
	}
}

func (q *QueryFfi) Delete() {
	C.query_free(q.ptr)
	q = nil
}

func (q QueryFfi) nextMessage() *C.char {
	return C.polar_next_query_message(q.ptr)
}

func (q QueryFfi) CallResult(callID uint64, term *types.Term) error {
	var s *C.char
	var err error
	if term != nil {
		s, err = ffiSerialize(term)
		defer C.free(unsafe.Pointer(s))
		if err != nil {
			return err
		}
	}

	result := C.polar_call_result(q.ptr, C.uint64_t(callID), s)
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) QuestionResult(callID uint64, answer bool) error {
	var intAnswer int
	if answer {
		intAnswer = 1
	} else {
		intAnswer = 0
	}
	result := C.polar_question_result(q.ptr, C.uint64_t(callID), C.int(intAnswer))
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) ApplicationError(message string) error {
	cMessage := C.CString(message)
	defer C.free(unsafe.Pointer(cMessage))
	result := C.polar_application_error(q.ptr, cMessage)
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) NextEvent() (*string, error) {
	event := C.polar_next_query_event(q.ptr)
	processMessages(q)
	if event == nil {
		return nil, getError()
	}
	goEvent := readStr(event)
	return &goEvent, nil
}

func (q QueryFfi) DebugCommand(command *string) error {
	term := types.Term{types.Value{types.ValueString(*command)}}
	cStr, err := ffiSerialize(term)
	defer C.free(unsafe.Pointer(cStr))
	if err != nil {
		return err
	}
	result := C.polar_debug_command(q.ptr, cStr)
	processMessages(q)
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) Source() (*string, error) {
	source := C.polar_query_source_info(q.ptr)
	if source == nil {
		return nil, getError()
	}
	goSource := readStr(source)
	return &goSource, nil
}
