// """Communicate with the Polar virtual machine: load rules, make queries, etc."""

package oso

import (
	"bufio"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"path/filepath"
	"reflect"

	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/internal/ffi"
	"github.com/osohq/go-oso/internal/host"
	"github.com/osohq/go-oso/internal/util"
	. "github.com/osohq/go-oso/types"
)

type Polar struct {
	ffiPolar ffi.PolarFfi
	host     host.Host
}

func newPolar() (*Polar, error) {
	ffiPolar := ffi.NewPolarFfi()
	polar := Polar{
		ffiPolar: ffiPolar,
		host:     host.NewHost(ffiPolar),
	}

	err := polar.registerConstant(host.None{}, "nil")
	if err != nil {
		return nil, err
	}

	builtinClasses := map[string]reflect.Type{
		"Boolean":    reflect.TypeOf(true),
		"Integer":    reflect.TypeOf(int(1)),
		"Float":      reflect.TypeOf(float64(1.0)),
		"String":     reflect.TypeOf(""),
		"List":       reflect.TypeOf(make([]interface{}, 0)),
		"Dictionary": reflect.TypeOf(make(map[string]interface{})),
	}

	for k, v := range builtinClasses {
		err := polar.registerClass(v, nil, &k)
		if err != nil {
			return nil, err
		}
	}

	// register global constants
	return &polar, nil
}

func (p Polar) checkInlineQueries() error {
	for {
		ffiQuery, err := p.ffiPolar.NextInlineQuery()
		if err != nil {
			return err
		}
		if ffiQuery == nil {
			return nil
		}
		query := newQuery(*ffiQuery, p.host.Copy())
		res, err := query.Next()
		if err != nil {
			return err
		}
		if res == nil {
			querySource, err := query.ffiQuery.Source()
			if err != nil {
				return err
			}
			return errors.NewInlineQueryFailedError(*querySource)
		}
	}
}

func (p Polar) loadFile(f string) error {
	if filepath.Ext(f) != ".polar" {
		return errors.NewPolarFileExtensionError(f)
	}

	data, err := ioutil.ReadFile(f)
	if err != nil {
		return err
	}
	err = p.ffiPolar.Load(string(data), &f)
	if err != nil {
		return err
	}
	return p.checkInlineQueries()
}

func (p Polar) loadString(s string) error {
	err := p.ffiPolar.Load(s, nil)
	if err != nil {
		return err
	}
	return p.checkInlineQueries()
}

func (p Polar) clearRules() error {
	return p.ffiPolar.ClearRules()
}

func (p Polar) queryStr(query string) (*Query, error) {
	ffiQuery, err := p.ffiPolar.NewQueryFromStr(query)
	if err != nil {
		return nil, err
	}
	newQuery := newQuery(*ffiQuery, p.host.Copy())
	return &newQuery, nil
}

func (p Polar) queryRule(name string, args ...interface{}) (*Query, error) {
	host := p.host.Copy()
	polarArgs := make([]Term, len(args))
	for idx, arg := range args {
		converted, err := host.ToPolar(arg)
		if err != nil {
			return nil, err
		}
		polarArgs[idx] = Term{*converted}
	}
	query := Call{
		Name: Symbol(name),
		Args: polarArgs,
	}
	inner := ValueCall(query)
	ffiQuery, err := p.ffiPolar.NewQueryFromTerm(Term{Value{inner}})
	if err != nil {
		return nil, err
	}
	newQuery := newQuery(*ffiQuery, host)
	return &newQuery, nil
}

func (p Polar) repl(files ...string) error {
	reader := bufio.NewReader(os.Stdin)
	for {
		fmt.Print("query> ")
		text, err := reader.ReadString('\n')
		if err == io.EOF {
			return nil
		}
		text = util.QueryStrip(text)

		ffiQuery, err := p.ffiPolar.NewQueryFromStr(text)
		if err != nil {
			fmt.Println(err)
			continue
		}
		query := newQuery(*ffiQuery, p.host.Copy())
		results, err := query.GetAllResults()
		if err != nil {
			fmt.Println(err)
			continue
		}
		if len(results) == 0 {
			fmt.Println(false)
		} else {
			for _, bindings := range results {
				if len(bindings) == 0 {
					fmt.Println(true)
				} else {
					for k, v := range bindings {
						switch v := v.(type) {
						// print strings with quotes but not variables or other types represented by strings
						case string:
							fmt.Printf("%v = %#v\n", k, v)
						default:
							fmt.Printf("%v = %v\n", k, v)
						}
					}
				}
			}
		}
	}
}

/*
Register a Go type with Polar so that it can be referenced within Polar files.
Accepts a concrete value of the Go type, a constructor function (or nil), and a
name (or nil).
*/
func (p Polar) registerClass(cls interface{}, ctor interface{}, name *string) error {
	// Get constructor
	constructor := reflect.ValueOf(nil)
	if ctor != nil {
		constructor = reflect.ValueOf(ctor)
		if constructor.Type().Kind() != reflect.Func {
			return fmt.Errorf("Constructor must be a function, got: %v", constructor.Type().Kind())
		}
	}

	// get real type
	var realType reflect.Type
	switch c := cls.(type) {
	case reflect.Type:
		realType = c
	default:
		realType = reflect.TypeOf(cls)
	}

	// Get class name
	var className string
	if name == nil {
		className = realType.Name()
	} else {
		className = *name
	}

	err := p.host.CacheClass(realType, className, constructor)
	if err != nil {
		return err
	}
	newVal := reflect.New(realType)
	return p.registerConstant(newVal.Interface(), className)
}

func (p Polar) registerConstant(value interface{}, name string) error {
	polarValue, err := p.host.ToPolar(value)
	if err != nil {
		return err
	}
	return p.ffiPolar.RegisterConstant(Term{*polarValue}, name)
}
