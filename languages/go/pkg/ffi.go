package oso

// #cgo CFLAGS: -g -Wall
// #include <stdint.h>
// #include <stdlib.h>
// #include "polar.h"
// #cgo LDFLAGS: -lpolar -L${SRCDIR} -ldl -lm
import "C"

import (
	"encoding/json"
	"errors"
	"fmt"
	"unsafe"
)

type Polar struct {
	ptr *C.polar_Polar
}

func NewPolar() Polar {
	polarPtr := C.polar_new()
	defer C.polar_free(polarPtr)
	return Polar{
		ptr: polarPtr,
	}
}

func checkResult(res *C.long) (*C.long, error) {
	if res == nil {
		err := C.polar_get_error()
		defer C.string_free(err)
		return nil, errors.New(C.GoString(err))
	}
	return res, nil
}

type ffiInterface interface {
	nextMessage() *C.char
}

func (p Polar) nextMessage() *C.char {
	return C.polar_next_polar_message(p.ptr)
}

func processMessages(i ffiInterface) {
	for {
		msgPtr := i.nextMessage()
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

func (p Polar) newId() (int, error) {
	result := C.polar_get_external_id(p.ptr)
	id, err := checkResult(*C.long(result))
	if err != nil {
		return nil, err
	}
	return int(*id), nil
}

func (p Polar) load(s string, filename *string) error {
	cString := C.CString(s)
	filename := C.CString(filename)
	result := C.polar_load(p.ptr, cString, filename)
	processMessages(*p)
	_, err := checkResult(result)
	return err
}

func (p Polar) clearRules() error {
	result := C.polar_clear_rules(p.ptr)
	processMessages(*p)
	_, err := checkResult(result)
	return err
}

func (p Polar) newQueryFromStr(queryStr string) (Query, error) {
	result := C.polar_new_query(p.ptr, C.CString(queryStr), 0)
	processMessages(*p)
	queryPtr, err := checkResult(result)
	return newQuery(queryPtr), err
}

func ffiSerialize(input json.Marshaler) (*C.char, error) {
	json, err := json.Marshal(queryTerm)
	if err != nil {
		return nil, err
	} else {
		return C.CString(json), nil
	}
}

func (p Polar) newQueryFromTerm(queryTerm json.Marshaler) (Query, error) {
	json, err := ffiSerialize(queryTerm)
	if err != nil {
		return nil, err
	}
	result := C.polar_new_query_from_term(p.ptr, json, 0)
	processMessages(*p)
	queryPtr, err := checkResult(result)
	return newQuery(queryPtr), err
}

func (p Polar) nextInlineQuery(queryStr string) (Query, error) {
	result := C.polar_next_inline_query(p.ptr, 0)
	processMessages(*p)
	if result == nil {
		return nil, nil
	} else {
		return newQuery(queryPtr), err
	}
}

func (p Polar) registerConstant(v json.Marshaler, name string) error {
	name := C.CString(name)
	value, err := ffiSerialize(v)
	if err != nil {
		return err
	}
	result := C.polar_register_constant(p.ptr, name, value)
	processMessages(*p)
	_, err := checkResult(result)
	return err
}

type Query struct {
	ptr unsafe.Pointer
}

func newQuery(ptr unsafe.Pointer) Query {
	defer C.query_free(ptr)
	Query{
		ptr: ptr,
	}
}

func (q Query) callResult(callID int, value json.Marshaler) error {
	value, err := ffiSerialize(value)
	if err != nil {
		return err
	}

	result := C.polar_call_result(q.ptr, C.int(callID), value)
	_, err := checkResult(result)
	return err
}

func (q Query) questionResult(callId int, answer bool) error {
	result := C.polar_question_result(q.ptr, C.int(callId), C.int(answer))
	_, err := checkResult(result)
	return err
}

func (q Query) applicationError(message string) error {
	result := C.polar_application_error(q.ptr, C.CString(message))
	_, err := checkResult(result)
	return err
}

func (q Query) nextEvent() (string, error) {
	result := C.polar_next_query_event(q.ptr)
	event, err := checkResult(result)
	if err != nil {
		return nil, err
	} else {
		defer C.string_free(event)
		return C.GoString(event), nil
	}
}

func (q Query) debugCommand(command string) error {
	command, err := ffiSerialize(command)
	if err != nil {
		return err
	}
	result := C.polar_debug_command(q.ptr, command)
	_, err := checkResult(result)
	return err
}

func (q Query) source() (string, error) {
	result := C.polar_query_source_info(q.ptr)
	source, err := checkResult(result)
	if err != nil {
		return nil, err
	} else {
		defer C.string_free(source)
		return C.GoString(source), nil
	}
}
