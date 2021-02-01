package ffi

// #cgo CFLAGS: -g -Wall
// #include <stdint.h>
// #include <stdlib.h>
// #include "native/polar.h"
// #cgo linux,amd64 LDFLAGS: ${SRCDIR}/native/linux/libpolar.a -ldl -lm
// #cgo darwin,amd64 LDFLAGS: ${SRCDIR}/native/macos/libpolar.a -ldl -lm
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"

	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/types"
)

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
	errStr := C.GoString(err)
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
		message := C.GoString(msgPtr)
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

func (p PolarFfi) load(s string, filename *string) error {
	cString := C.CString(s)
	var cFilename *C.char
	if filename != nil {
		cFilename = C.CString(*filename)
	}
	result := C.polar_load(p.ptr, cString, cFilename)
	processMessages(p)
	if result == 0 {
		return getError()
	}
	return nil
}

func (p PolarFfi) clearRules() error {
	result := C.polar_clear_rules(p.ptr)
	processMessages(p)
	if result == 0 {
		return getError()
	}
	return nil
}

func (p PolarFfi) newQueryFromStr(queryStr string) (*QueryFfi, error) {
	cs := C.CString(queryStr)
	result := C.polar_new_query(p.ptr, cs, 0)
	processMessages(p)
	if result == nil {
		return nil, getError()
	}
	return newQueryFfi(result), nil
}

func (p PolarFfi) newQueryFromTerm(queryTerm types.Term) (*QueryFfi, error) {
	json, err := ffiSerialize(queryTerm)
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

func (p PolarFfi) nextInlineQuery() (*QueryFfi, error) {
	queryPtr := C.polar_next_inline_query(p.ptr, 0)
	processMessages(p)
	if queryPtr == nil {
		// TODO: we don't have any way of signaling this failing?
		return nil, nil
	}
	return newQueryFfi(queryPtr), nil
}

func (p PolarFfi) registerConstant(term types.Term, name string) error {
	cName := C.CString(name)
	cTerm, err := ffiSerialize(term)
	if err != nil {
		defer C.free(unsafe.Pointer(cName))
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

func (q *QueryFfi) delete() {
	C.query_free(q.ptr)
	q = nil
}

func (q QueryFfi) nextMessage() *C.char {
	return C.polar_next_query_message(q.ptr)
}

func (q QueryFfi) callResult(callID uint64, term *types.Term) error {
	var s *C.char
	var err error
	if term != nil {
		s, err = ffiSerialize(term)
		if err != nil {
			return err
		}
	}

	result := C.polar_call_result(q.ptr, C.__uint64_t(callID), s)
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) questionResult(callID uint64, answer bool) error {
	var intAnswer int
	if answer {
		intAnswer = 1
	} else {
		intAnswer = 0
	}
	result := C.polar_question_result(q.ptr, C.__uint64_t(callID), C.int(intAnswer))
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) applicationError(message string) error {
	result := C.polar_application_error(q.ptr, C.CString(message))
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) nextEvent() (*string, error) {
	event := C.polar_next_query_event(q.ptr)
	processMessages(q)
	if event == nil {
		return nil, getError()
	}
	goEvent := C.GoString(event)
	return &goEvent, nil
}

func (q QueryFfi) debugCommand(command *string) error {
	cStr, err := ffiSerialize(command)
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

func (q QueryFfi) source() (*string, error) {
	source := C.polar_query_source_info(q.ptr)
	if source == nil {
		return nil, getError()
	}
	goSource := C.GoString(source)
	return &goSource, nil
}
