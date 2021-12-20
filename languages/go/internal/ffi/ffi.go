package ffi

// #cgo CFLAGS: -g -Wall
// #include <stdint.h>
// #include <stdlib.h>
// #include "native/polar.h"
// #cgo linux,amd64 LDFLAGS: ${SRCDIR}/native/linux/libpolar.a -ldl -lm
// #cgo darwin,amd64 LDFLAGS: ${SRCDIR}/native/macos/amd64/libpolar.a -ldl -lm
// #cgo darwin,arm64 LDFLAGS: ${SRCDIR}/native/macos/arm64/libpolar.a -ldl -lm
// #cgo windows,amd64 LDFLAGS: ${SRCDIR}/native/windows/libpolar.a -lm -lws2_32 -luserenv -lbcrypt
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"

	"github.com/osohq/go-oso/errors"
	_ "github.com/osohq/go-oso/internal/ffi/native"
	"github.com/osohq/go-oso/types"
)

/*
Reads a c string from polar core to a go string and frees the c string.
*/
func readStr(cStr *C.char) *string {
	if cStr == nil {
		return nil
	}
	goStr := C.GoString(cStr)
	C.string_free(cStr)
	return &goStr
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

/*
The checkResult{Void,String,Query} methods  take the
result _pointer_ returned from Rust, extract the result
and turn it into (T, error) format, and free the memory
used by the result.

Results are of the form struct { T result, string error }
`result_free` only frees those two pointers. The error
is freed by calling getError (which calls readStr),
and the pointer is (a) void, (b) freed in calling readStr,
or (c) wrapped in a QueryFfi struct, depending on the method
*/

func checkResultVoid(res *C.polar_CResult_c_void) error {
	err := res.error
	resultPtr := res.result
	defer C.result_free(res)
	if err != nil {
		if resultPtr != nil {
			panic("Internal error: both result and error pointers are not null")
		}
		return getError(err)
	}
	return nil
}

func checkResultString(res *C.polar_CResult_c_char) (*string, error) {
	err := res.error
	resultPtr := res.result
	// it's fine to cast this pointer to result c_void, since Rust wont
	// do anything with inner pointers anyway
	defer C.result_free((*C.polar_CResult_c_void)((unsafe.Pointer)(res)))
	if err != nil {
		if resultPtr != nil {
			panic("Internal error: both result and error pointers are not null")
		}
		return nil, getError(err)
	}
	result := readStr(resultPtr)
	return result, nil
}

func checkResultQuery(res *C.polar_CResult_Query) (*QueryFfi, error) {
	err := res.error
	resultPtr := res.result
	// it's fine to cast this pointer to result c_void, since Rust wont
	// do anything with inner pointers anyway
	defer C.result_free((*C.polar_CResult_c_void)((unsafe.Pointer)(res)))
	if err != nil {
		if resultPtr != nil {
			panic("Internal error: both result and error pointers are not null")
		}
		return nil, getError(err)
	}
	result := newQueryFfi(resultPtr)
	return result, nil
}

func getError(err *C.char) error {
	errStr := *readStr(err)
	var polarError errors.FormattedPolarError
	jsonErr := json.Unmarshal([]byte(errStr), &polarError)
	if jsonErr != nil {
		return jsonErr
	}
	return &polarError
}

type ffiInterface interface {
	nextMessage() (*string, error)
}

func (p PolarFfi) nextMessage() (*string, error) {
	return checkResultString(C.polar_next_polar_message(p.ptr))
}

func processMessages(i ffiInterface) {
	for {
		message, err := i.nextMessage()
		if err != nil {
			panic(err)
		}
		if message == nil {
			return
		}
		var messageStruct types.Message
		err = json.Unmarshal([]byte(*message), &messageStruct)

		if err != nil {
			panic(err)
		}
		switch messageStruct.Kind.MessageKindVariant.(type) {
		case types.MessageKindPrint:
			fmt.Printf("%s\n", messageStruct.Msg)
		case types.MessageKindWarning:
			fmt.Printf("WARNING: %s\n", messageStruct.Msg)
		default:
			fmt.Printf("Unexpected message: %#v\n", messageStruct)
		}
	}
}

func (p PolarFfi) NewId() (uint64, error) {
	id := C.polar_get_external_id(p.ptr)
	return uint64(id), nil
}

func (p PolarFfi) Load(sources []types.Source) error {
	json, err := ffiSerialize(sources)
	defer C.free(unsafe.Pointer(json))
	if err != nil {
		return err
	}
	err = checkResultVoid(C.polar_load(p.ptr, json))
	processMessages(p)
	return err
}

func (p PolarFfi) ClearRules() error {
	err := checkResultVoid(C.polar_clear_rules(p.ptr))
	processMessages(p)
	return err
}

func (p PolarFfi) NewQueryFromStr(queryStr string) (*QueryFfi, error) {
	cs := C.CString(queryStr)
	defer C.free(unsafe.Pointer(cs))
	result, err := checkResultQuery(C.polar_new_query(p.ptr, cs, 0))
	processMessages(p)
	return result, err
}

func (p PolarFfi) NewQueryFromTerm(queryTerm types.Term) (*QueryFfi, error) {
	json, err := ffiSerialize(queryTerm)
	defer C.free(unsafe.Pointer(json))
	if err != nil {
		return nil, err
	}
	result, err := checkResultQuery(C.polar_new_query_from_term(p.ptr, json, 0))
	processMessages(p)
	return result, err
}

func (p PolarFfi) NextInlineQuery() *QueryFfi {
	queryPtr := C.polar_next_inline_query(p.ptr, 0)
	processMessages(p)
	return newQueryFfi(queryPtr)
}

func (p PolarFfi) RegisterConstant(term types.Term, name string) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cTerm, err := ffiSerialize(term)
	defer C.free(unsafe.Pointer(cTerm))
	if err != nil {
		return err
	}
	err = checkResultVoid(C.polar_register_constant(p.ptr, cName, cTerm))
	processMessages(p)
	return err
}

func (p PolarFfi) RegisterMro(name string, mro []uint64) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cMro, err := ffiSerialize(mro)
	defer C.free(unsafe.Pointer(cMro))
	if err != nil {
		return err
	}
	err = checkResultVoid(C.polar_register_mro(p.ptr, cName, cMro))
	processMessages(p)
	return err
}

// yeah, not ideal types yet lol
func (p PolarFfi) BuildDataFilter(user_types map[string]map[string]map[string]map[string]string, partials []map[string]map[string]types.Term, resource_var string, resource_type string) (*types.Filter, error) {
	cTypes, err := ffiSerialize(user_types)
	defer C.free(unsafe.Pointer(cTypes))
	if err != nil {
		return nil, err
	}
	cPartials, err := ffiSerialize(partials)
	defer C.free(unsafe.Pointer(cPartials))
	if err != nil {
		return nil, err
	}
	cVar := C.CString(resource_var)
	defer C.free(unsafe.Pointer(cVar))
	cType := C.CString(resource_type)
	defer C.free(unsafe.Pointer(cType))
	filterJson, err := checkResultString(C.polar_build_data_filter(p.ptr, cTypes, cPartials, cVar, cType))
	if err != nil {
		return nil, err
	}

	processMessages(p)

	var filter types.Filter
	err = json.Unmarshal([]byte(*filterJson), &filter)
	if err != nil {
		return nil, err
	}

	return &filter, err
}

type QueryFfi struct {
	ptr *C.polar_Query
}

func newQueryFfi(ptr *C.polar_Query) *QueryFfi {
	if ptr == nil {
		return nil
	}
	return &QueryFfi{
		ptr: ptr,
	}
}

func (q *QueryFfi) Delete() {
	C.query_free(q.ptr)
	q = nil
}

func (q QueryFfi) nextMessage() (*string, error) {
	return checkResultString(C.polar_next_query_message(q.ptr))
}

func (q QueryFfi) CallResult(callID uint64, term *types.Term) error {
	var s *C.char
	var err error
	s, err = ffiSerialize(term)
	defer C.free(unsafe.Pointer(s))
	if err != nil {
		return err
	}

	return checkResultVoid(C.polar_call_result(q.ptr, C.uint64_t(callID), s))
}

func (q QueryFfi) QuestionResult(callID uint64, answer bool) error {
	var intAnswer int
	if answer {
		intAnswer = 1
	} else {
		intAnswer = 0
	}
	return checkResultVoid(C.polar_question_result(q.ptr, C.uint64_t(callID), C.int(intAnswer)))
}

func (q QueryFfi) ApplicationError(message string) error {
	cMessage := C.CString(message)
	defer C.free(unsafe.Pointer(cMessage))
	return checkResultVoid(C.polar_application_error(q.ptr, cMessage))
}

func (q QueryFfi) NextEvent() (*string, error) {
	result, err := checkResultString(C.polar_next_query_event(q.ptr))
	processMessages(q)
	return result, err
}

func (q QueryFfi) DebugCommand(command *string) error {
	term := types.Term{types.Value{types.ValueString(*command)}}
	cStr, err := ffiSerialize(term)
	defer C.free(unsafe.Pointer(cStr))
	if err != nil {
		return err
	}
	err = checkResultVoid(C.polar_debug_command(q.ptr, cStr))
	processMessages(q)
	return err
}

func (q QueryFfi) Source() (*string, error) {
	return checkResultString(C.polar_query_source_info(q.ptr))
}

func (q QueryFfi) Bind(name string, value *types.Term) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	var s *C.char
	//var err error
	s, err := ffiSerialize(value)
	defer C.free(unsafe.Pointer(s))
	if err != nil {
		return err
	}
	err = checkResultVoid(C.polar_bind(q.ptr, cName, s))
	processMessages(q)
	return err
}
