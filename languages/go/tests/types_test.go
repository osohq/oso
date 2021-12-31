package oso_test

import (
	"encoding/json"
	"fmt"
	"reflect"
	"testing"

	"github.com/google/go-cmp/cmp"
	. "github.com/osohq/go-oso/types"
)

func TestSerialize(t *testing.T) {
	expected := `{"value":{"Number":{"Integer":123}}}`
	var term Term
	term = Term{ValueNumber{
		NumericInteger(123),
	}}
	s, err := json.Marshal(term)
	if err != nil {
		t.Fatal(err)
	}
	if string(s) != expected {
		t.Fatal(fmt.Errorf("expected %#v got %#v", expected, string(s)))
	}

	expected = `{"value":{"Expression":{"operator":"Add","args":[{"value":{"Number":{"Integer":123}}},{"value":{"Number":{"Integer":234}}}]}}}`
	term = Term{ValueExpression{
		Operator: OperatorAdd{},
		Args: []Term{
			{ValueNumber{
				NumericInteger(123),
			}},
			{ValueNumber{
				NumericInteger(234),
			}},
		},
	}}
	s, err = json.Marshal(term)
	if err != nil {
		t.Fatal(err)
	}
	if string(s) != expected {
		t.Fatal(fmt.Errorf("expected %#v got %#v. Diff: %s", expected, string(s), cmp.Diff(expected, string(s))))
	}

}

func TestDeserialize(t *testing.T) {
	jsonTerm := []byte(`{
		"value": {
			"Call": {
				"name": "foo",
				"args": [{"value": {"Number": {"Integer": 0}}}],
				"kwargs": {"bar": {"value": {"Number": {"Integer": 1}}}}
			}
		}
	}`)

	var term Term
	err := json.Unmarshal(jsonTerm, &term)
	if err != nil {
		t.Fatal(err)
	}
	kwargs := make(map[Symbol]Term)
	kwargs[Symbol("bar")] = Term{ValueNumber{NumericInteger(1)}}
	expectedCall := Term{ValueCall{
		Name: "foo",
		Args: []Term{{
			ValueNumber{NumericInteger(0)}},
		},
		Kwargs: &kwargs,
	}}
	expected := expectedCall
	if !cmp.Equal(term, expected) || !reflect.DeepEqual(term, expected) {
		t.Error(fmt.Errorf("Result differs from expected:\n%s", cmp.Diff(term, expected)))
	}

	jsonErrTerm := []byte(`{"kind":{"Parse":{"InvalidTokenCharacter":{"token":"this is not","c":"\n","loc":24}}},"formatted":"'\\n' is not a valid character. Found in this is not at line 1, column 25"}`)
	var errTerm FormattedPolarError
	err = json.Unmarshal(jsonErrTerm, &errTerm)
	if err != nil {
		t.Fatal(err)
	}
	expectedErr := FormattedPolarError{
		Kind: ErrorKindParse{
			ParseErrorInvalidTokenCharacter{
				Token: "this is not",
				C:     "\n",
				Loc:   24,
			},
		},
		Formatted: "'\\n' is not a valid character. Found in this is not at line 1, column 25",
	}
	if !cmp.Equal(errTerm, expectedErr) {
		t.Error(fmt.Errorf("Result differs from expected:\n%s", cmp.Diff(errTerm, expectedErr)))
	}
}
