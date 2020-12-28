package oso

import (
	"encoding/json"
	"fmt"
	"reflect"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestSerialize(t *testing.T) {
	expected := `{"Number":{"Integer":123}}`
	int123 := NumericInteger(123)
	term := Value{&ValueNumber{
		&int123,
	}}
	s, err := json.Marshal(term)
	if err != nil {
		t.Fatal(err)
	}
	if string(s) != expected {
		t.Fatal(fmt.Errorf("expected %#v got %#v", expected, string(s)))
	}

}

func TestDeserialize(t *testing.T) {
	jsonTerm := []byte(`{
        "Call": {
            "name": "foo",
            "args": [{"Number": {"Integer": 0}}],
            "kwargs": {"bar": {"Number": {"Integer": 1}}}
        }
	}`)

	var term Value
	err := json.Unmarshal(jsonTerm, &term)
	if err != nil {
		t.Fatal(err)
	}
	int0 := NumericInteger(0)
	int1 := NumericInteger(1)
	expectedCall := ValueCall{
		Name:   "foo",
		Args:   []Value{Value{&ValueNumber{&int0}}},
		Kwargs: &map[string]Value{"bar": Value{&ValueNumber{&int1}}},
	}
	expected := Value{&expectedCall}
	if !cmp.Equal(term, expected) || !reflect.DeepEqual(term, expected) {
		t.Error(fmt.Errorf("Result differs from expected:\n%s", cmp.Diff(term, expected)))
	}
}
