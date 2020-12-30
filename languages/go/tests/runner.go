package oso

import (
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

var CLASSES = map[string]reflect.Type{
	"UnitClass":       reflect.TypeOf(UnitClass{}),
	"ValueFactory":    reflect.TypeOf(ValueFactory{}),
	"IterableClass":   reflect.TypeOf(IterableClass{}),
	"Constructor":     reflect.TypeOf(Constructor{}),
	"MethodVariants":  reflect.TypeOf(MethodVariants{}),
	"ParentClass":     reflect.TypeOf(ParentClass{}),
	"ChildClass":      reflect.TypeOf(ChildClass{}),
	"GrandchildClass": reflect.TypeOf(GrandchildClass{}),
	"Animal":          reflect.TypeOf(Animal{}),
	"ImplementsEq":    reflect.TypeOf(ImplementsEq{}),
	"Comparable":      reflect.TypeOf(Comparable{}),
}

type TestCase struct {
	// Raw         string                   `yaml:omit`
	Name        string   `yaml:"name"`
	Description string   `yaml:"description"`
	Policies    []string `yaml:"policies"`
	Cases       []Case   `yaml:"cases"`
}

type Result struct {
	inner interface{}
}

func toResult(input map[string]interface{}) map[string]Result {
	result := make(map[string]Result)
	for k, v := range input {
		result[k] = NewResult(v)
	}
	return result
}

func NewResult(input interface{}) Result {
	switch inputVal := input.(type) {
	case map[string]interface{}:
		result := make(map[string]Result)
		for k, v := range inputVal {
			result[k] = NewResult(v)
		}
		return Result{result}
	case []interface{}:
		result := make([]Result, len(inputVal))
		for idx, v := range inputVal {
			result[idx] = NewResult(v)
		}
		return Result{result}
	case uint64:
		// standardise uints to ints
		return Result{inner: int(inputVal)}
	default:
		return Result{input}
	}
}

func (left Result) Equal(right interface{}) bool {
	switch val := left.inner.(type) {
	case map[string]Result:
		if repr, ok := val["repr"]; ok {
			fmt.Printf("%v", right)
			return repr.inner.(string) == fmt.Sprintf("%v", right)
		}
	}
	fmt.Printf("%v == %v: %v", left.inner, right, cmp.Equal(left.inner, right))
	return cmp.Equal(left.inner, right)
}

type Variable string

func toInput(o oso.Polar, v interface{}, t *testing.T) interface{} {
	if vMap, ok := v.(map[string]interface{}); ok {
		if ty, ok := vMap["type"]; ok {
			class := CLASSES[ty.(string)]
			if class == nil {
				t.Fatalf("class %s not implemented for tests", ty.(string))
			}
			var args []interface{}
			var kwargs map[string]interface{}
			if vMap["args"] != nil {
				args = vMap["args"].([]interface{})
			}
			if vMap["kwargs"] != nil {
				kwargs = vMap["kwargs"].(map[string]interface{})
			}
			instance, err := oso.InstantiateClass(class, args, kwargs)
			if err != nil {
				t.Fatal(err)
			}
			return instance
		}
		if v, ok := vMap["var"]; ok {
			return Variable(v.(string))
		}
	}
	return v
}

type Case struct {
	Description *string                   `yaml:"description"`
	Query       string                    `yaml:"query"`
	Load        *string                   `yaml:"load"`
	Inputs      *[]interface{}            `yaml:"input"`
	Result      *[]map[string]interface{} `yaml:"result"`
	Err         *string                   `yaml:"err"`
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

func (tc TestCase) setupTest(o oso.Polar, t *testing.T) error {
	for k, v := range CLASSES {
		err := o.RegisterClass(v, &k)
		if err != nil {
			t.Fatal(err)
		}
	}

	err := filepath.Walk("../../../test/policies/", func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}

		if contains(tc.Policies, strings.TrimSuffix(filepath.Base(path), filepath.Ext(path))) {
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

func (tc TestCase) RunTest(t *testing.T) {
	for _, c := range tc.Cases {
		testName := tc.Name + "\n" + tc.Description + "\n"
		if c.Description != nil {
			testName += *c.Description
		} else {
			testName += c.Query
		}
		t.Run(testName, func(t *testing.T) {
			o := *oso.NewPolar()
			err := tc.setupTest(o, t)
			if err != nil {
				t.Fatal(err)
			}
			var testQuery *oso.Query
			var queryErr error
			if c.Inputs == nil {
				testQuery, queryErr = o.Query(c.Query)
			} else {
				Inputs := make([]interface{}, len(*c.Inputs))
				for idx, v := range *c.Inputs {
					input := toInput(o, v, t)
					Inputs[idx] = input
				}
				testQuery, queryErr = o.QueryRule(c.Query, Inputs...)
			}

			expectedResults := make([]map[string]Result, 0)
			if c.Result == nil {
				expectedResults = append(expectedResults, toResult(make(map[string]interface{})))
			} else {
				for _, v := range *c.Result {
					expectedResults = append(expectedResults, toResult(v))
				}
			}

			if c.Load != nil {
				o.LoadString(*c.Load)
			}

			results := make([]map[string]interface{}, 0)
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
					results = append(results, *v)
				}
			}
			if c.Err != nil {
				if queryErr != nil {
					re, err := regexp.Compile(*c.Err)
					if err != nil {
						t.Error(err)
					}
					if !re.Match([]byte(queryErr.Error())) {
						t.Error(fmt.Errorf("expected query to fail with:\n\t\"%s\"\nGot:\n\t\"%s\"", *c.Err, queryErr))
					}
				} else {
					t.Error(fmt.Errorf("expected query to fail with:\n\t\"%s\". Got:\n\tSuccess", *c.Err))
				}
			} else {
				if queryErr != nil {
					t.Error(queryErr)
				} else {
					if len(results) != len(expectedResults) {
						t.Error(fmt.Errorf("incorrect query result:\n%s", cmp.Diff(expectedResults, results)))
						return
					}
					for idx, expectedResult := range expectedResults {
						result := results[idx]
						for k, v := range expectedResult {
							if v2, ok := result[k]; ok {
								if !v.Equal(v2) {
									t.Error(fmt.Errorf("incorrect query result:\n%s", cmp.Diff(v2, v)))
								}
							} else {
								t.Error(fmt.Errorf("incorrect query result:\n%s", cmp.Diff(v, nil)))
							}
						}
					}
				}

			}
		})
	}
}
