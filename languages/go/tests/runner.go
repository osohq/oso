package oso

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"regexp"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	oso "github.com/osohq/oso/languages/go/pkg"
)

type TestCase struct {
	// Raw         string                   `yaml:omit`
	Name        string   `yaml:"name"`
	Description string   `yaml:"description"`
	Policies    []string `yaml:"policies"`
	Cases       []Case   `yaml:"cases"`
}

type Result map[string]interface{}

func toResult(input interface{}) interface{} {
	switch input.(type) {
	case map[string]interface{}:
		return NewResult(input.(map[string]interface{}))
	default:
		return input
	}
}

func NewResult(input map[string]interface{}) Result {
	result := make(map[string]interface{})
	for k, v := range input {
		result[k] = toResult(v)
	}
	return result
}

func (left Result) Equal(right interface{}) bool {
	if val, ok := left["repr"]; ok {
		return reflect.DeepEqual(val, right)
	} else {
		return reflect.DeepEqual(left, right)
	}
}

type Variable string

func ToInput(v interface{}) (interface{}, error) {
	switch v.(type) {
	case map[string]interface{}:
		if _, ok := v.(map[string]interface{})["type"]; ok {
			return nil, errors.New("classes aren't supported yet")
		}
		if v, ok := v.(map[string]interface{})["var"]; ok {
			return Variable(v.(string)), nil
		}
	}
	return v, nil
}

// def to_input(v):
//     if isinstance(v, dict):
//         if "type" in v:
//             cls = getattr(classes, v["type"])
//             args = [to_input(v) for v in v.get("args", [])]
//             kwargs = {k: to_input(v) for k, v in v.get("kwargs", {}).items()}
//             return cls(*args, **kwargs)
//         elif "var" in v:
//             return Variable(v["var"])
//     return v

// type TestCase struct {
// 	Name        string      `yaml:"name"`
// 	Description string      `yaml:"description"`
// 	Policies    []string    `yaml:"policies"`
// 	Case        interface{} `yaml:"cases"`
// }

type Case struct {
	Description *string                  `yaml:"description"`
	Query       string                   `yaml:"query"`
	Load        *string                  `yaml:"load"`
	Inputs      *[]interface{}           `yaml:"input"`
	Result      []map[string]interface{} `yaml:"result"`
	Err         *string                  `yaml:"err"`
}

func contains(list []string, elem string) bool {
	for _, a := range list {
		if a == elem {
			return true
		}
	}
	return false
}

func String(s string) *string {
	return &s
}

func (tc TestCase) setupTest(o *oso.Polar, t *testing.T) error {
	err := o.RegisterClass(reflect.TypeOf(ValueFactory{}), String("ValueFactory"))
	if err != nil {
		t.Fatal(err)
	}
	err = o.RegisterClass(reflect.TypeOf(UnitClass{}), String("UnitClass"))
	if err != nil {
		t.Fatal(err)
	}

	err = filepath.Walk("../../../test/policies/", func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}

		if contains(tc.Policies, strings.TrimSuffix(path, filepath.Ext(path))) {
			err = o.LoadFile(path)
			if err != nil {
				return err
			}
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	return nil
}

func (tc TestCase) RunTest(o *oso.Polar, t *testing.T) {
	err := tc.setupTest(o, t)
	if err != nil {
		t.Fatal(err)
	}
	for _, c := range tc.Cases {
		testName := tc.Name + "\n" + tc.Description + "\n"
		if c.Description != nil {
			testName += *c.Description
		} else {
			testName += c.Query
		}
		t.Run(testName, func(t *testing.T) {
			var testQuery *oso.Query
			var queryErr error
			if c.Inputs == nil {
				// fmt.Printf("Querying for: %s", c.Query)
				testQuery, queryErr = o.Query(c.Query)
			} else {
				Inputs := make([]interface{}, len(*c.Inputs))
				for idx, v := range *c.Inputs {
					input, err := ToInput(v)
					if err != nil {
						t.Error(err)
					}
					Inputs[idx] = input
				}
				// fmt.Printf("Querying for: %s(%v)", c.Query, Inputs)
				testQuery, queryErr = o.QueryRule(c.Query, Inputs...)
			}

			expectedResults := make([]Result, len(c.Result))
			for idx, v := range c.Result {
				expectedResults[idx] = NewResult(v)
			}

			if c.Load != nil {
				o.LoadString(*c.Load)
			}

			results := make([]Result, 0)
			if queryErr == nil {
				for {
					v, err := testQuery.Next()
					if err != nil {
						queryErr = err
						break
					}
					if v == nil {
						break
					}
					results = append(results, NewResult(*v))
				}
			}
			if c.Err != nil {
				if queryErr != nil {
					re, err := regexp.Compile(*c.Err)
					if err != nil {
						t.Error(err)
					}
					if !re.Match([]byte(queryErr.Error())) {
						t.Error(fmt.Errorf("expected query to fail with %s. Got %s", *c.Err, queryErr))
					}
				} else {
					t.Error(fmt.Errorf("expected query to fail with %s. Got success", *c.Err))
				}
			} else {
				if queryErr != nil {
					t.Error(queryErr)
				} else {
					if !cmp.Equal(results, expectedResults) {
						t.Error(fmt.Errorf("unexpected query result:\n%s", cmp.Diff(results, expectedResults)))
					}
				}

			}
		})
	}
}
