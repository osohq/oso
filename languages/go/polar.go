// """Communicate with the Polar virtual machine: load rules, make queries, etc."""

package oso

import (
	"fmt"
	"io/ioutil"
	"path/filepath"
	"reflect"

	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/internal/ffi"
	"github.com/osohq/go-oso/internal/host"
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
		err := polar.registerClass(v, &k)
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
	polarArgs := make([]Term, len(args))
	for idx, arg := range args {
		converted, err := p.host.ToPolar(arg)
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
	newQuery := newQuery(*ffiQuery, p.host.Copy())
	return &newQuery, nil
}

func (p Polar) repl(files ...string) error {
	return fmt.Errorf("Go REPL is not yet implemented")
}

func (p Polar) registerClass(cls reflect.Type, name *string) error {
	var className string
	if name == nil {
		className = cls.Name()
	} else {
		className = *name
	}

	err := p.host.CacheClass(cls, className)
	if err != nil {
		return err
	}
	newVal := reflect.New(cls)
	return p.registerConstant(newVal.Interface(), className)
}

func (p Polar) registerConstant(value interface{}, name string) error {
	polarValue, err := p.host.ToPolar(value)
	if err != nil {
		return err
	}
	return p.ffiPolar.RegisterConstant(Term{*polarValue}, name)
}
