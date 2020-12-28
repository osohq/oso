package oso

import (
	"fmt"
	"reflect"
	"testing"
)

func TestJson(t *testing.T) {
	json_term := []byte(`{
        "Call": {
            "name": "foo",
            "args": [{"Number": {"Integer": 0}}],
            "kwargs": {"bar": {"Number": {"Integer": 1}}}
        }
	}`)

	term, err := DeserializeValue(json_term)
	if err != nil {
		t.Fatal(err)
	}
	int0 := Numeric__Integer(0)
	int1 := Numeric__Integer(1)
	expected := &Value__Call{
		Value: Call{
			Name:   "foo",
			Args:   []Value{&Value__Number{Value: &int0}},
			Kwargs: &map[string]Value{"bar": &Value__Number{Value: &int1}},
		},
	}
	if !reflect.DeepEqual(term, expected) {
		t.Error(fmt.Errorf("expected %#v got %#v", expected, term))
	}
}
