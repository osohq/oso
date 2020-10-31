package oso

// #cgo CFLAGS: -g -Wall
// #include <stdint.h>
// #include <stdlib.h>
// #include "polar.h"
// #cgo LDFLAGS: -lpolar -L${SRCDIR} -ldl -lm
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"
)

// DoThing Does a thing
func DoThing() {

	polar := C.polar_new()
	queryStr := C.CString("x = 1 + 1")
	query := C.polar_new_query(polar, queryStr, 0)
	defer C.free(unsafe.Pointer(queryStr))
	resultStr := []byte(C.GoString(C.polar_next_query_event(query)))
	var result QueryEvent
	err := json.Unmarshal(resultStr, &result)
	if err != nil {
		fmt.Println("error:", err)
	}
	switch result.Kind {
	case "Result":
		bindings := result.Data["bindings"]
		original, _ := bindings.(map[string]Term)
		fmt.Println(original)
	default:
		fmt.Println("Unsupported event: ", result.Kind)
	}

}
