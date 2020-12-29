package oso

// #cgo CFLAGS: -g -Wall
// #include <stdint.h>
// #include <stdlib.h>
// #include "../../../polar-c-api/polar.h"
// #cgo LDFLAGS: ${SRCDIR}/../../../target/debug/libpolar.a -ldl -lm
import "C"

import (
	"encoding/json"
	"fmt"
)

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
	defer C.string_free(err)
	errStr := C.GoString(err)
	var polarError FormattedPolarError
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
		defer C.string_free(msgPtr)
		var messageStruct struct {
			kind string
			msg  string
		}
		err := json.Unmarshal([]byte(message), &messageStruct)
		if err != nil {
			continue
		}
		switch messageStruct.kind {
		case "Print":
			fmt.Print(messageStruct.msg)
			break
		case "Warning":
			fmt.Printf("WARNING: %s", messageStruct.msg)
			break
		}
	}
}

func (p PolarFfi) newId() (int, error) {
	id := C.polar_get_external_id(p.ptr)
	if id == 0 {
		return 0, getError()
	}
	return int(id), nil
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
	result := C.polar_new_query(p.ptr, C.CString(queryStr), C.uint(0))
	processMessages(p)
	if result == nil {
		return nil, getError()
	}
	return newQueryFfi(result), nil
}

func ffiSerialize(input interface{}) (*C.char, error) {
	json, err := json.Marshal(input)
	if err != nil {
		return nil, err
	}
	return C.CString(string(json)), nil
}

func (p PolarFfi) newQueryFromTerm(queryTerm interface{}) (*QueryFfi, error) {
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

func (p PolarFfi) registerConstant(v Value, name string) error {
	cName := C.CString(name)
	cValue, err := ffiSerialize(v)
	if err != nil {
		return err
	}
	result := C.polar_register_constant(p.ptr, cName, cValue)
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

func (q QueryFfi) callResult(callID int, value Value) error {
	s, err := ffiSerialize(value)
	if err != nil {
		return err
	}

	result := C.polar_call_result(q.ptr, C.ulong(callID), s)
	if result == 0 {
		return getError()
	}
	return nil
}

func (q QueryFfi) questionResult(callID int, answer bool) error {
	var intAnswer int
	if answer {
		intAnswer = 1
	} else {
		intAnswer = 0
	}
	result := C.polar_question_result(q.ptr, C.ulong(callID), C.int(intAnswer))
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
	if event == nil {
		return nil, getError()
	}
	defer C.string_free(event)
	goEvent := C.GoString(event)
	return &goEvent, nil
}

func (q QueryFfi) debugCommand(command interface{}) error {
	cStr, err := ffiSerialize(command)
	if err != nil {
		return err
	}
	result := C.polar_debug_command(q.ptr, cStr)
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
	defer C.string_free(source)
	goSource := C.GoString(source)
	return &goSource, nil
}
