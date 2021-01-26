// """Communicate with the Polar virtual machine: load rules, make queries, etc."""

package oso

import (
	"fmt"
	"io/ioutil"
	"path/filepath"
	"reflect"
)

var CLASSES = make(map[string]reflect.Type)

type Polar struct {
	ffiPolar PolarFfi
	host     Host
}

type none struct{}

func NewPolar() *Polar {
	ffiPolar := NewPolarFfi()
	polar := Polar{
		ffiPolar: ffiPolar,
		host:     NewHost(ffiPolar),
	}

	err := polar.RegisterConstant(none{}, "nil")
	if err != nil {
		fmt.Printf(err.Error())
		return nil
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
		err := polar.RegisterClass(v, &k)
		if err != nil {
			fmt.Printf(err.Error())
			return nil
		}
	}

	// register global constants
	return &polar
}

func (p Polar) checkInlineQueries() error {
	for {
		ffiQuery, err := p.ffiPolar.nextInlineQuery()
		if err != nil {
			return err
		}
		if ffiQuery == nil {
			return nil
		}
		query := newQuery(*ffiQuery, p.host.copy())
		res, err := query.Next()
		if err != nil {
			return err
		}
		if res == nil {
			querySource, err := query.ffiQuery.source()
			if err != nil {
				return err
			}
			return &InlineQueryFailedError{source: *querySource}
		}
	}
}

func (p Polar) LoadFile(f string) error {
	if filepath.Ext(f) != ".polar" {
		return &PolarFileExtensionError{file: f}
	}

	data, err := ioutil.ReadFile(f)
	if err != nil {
		return err
	}
	err = p.ffiPolar.load(string(data), &f)
	if err != nil {
		return err
	}
	return p.checkInlineQueries()
}

func (p Polar) LoadString(s string) error {
	err := p.ffiPolar.load(s, nil)
	if err != nil {
		return err
	}
	return p.checkInlineQueries()
}

func (p Polar) ClearRules() error {
	return p.ffiPolar.clearRules()
}

func (p Polar) QueryStr(query string) (*Query, error) {
	ffiQuery, err := p.ffiPolar.newQueryFromStr(query)
	if err != nil {
		return nil, err
	}
	newQuery := newQuery(*ffiQuery, p.host.copy())
	return &newQuery, nil
}

func (p Polar) QueryRule(name string, args ...interface{}) (*Query, error) {
	polarArgs := make([]Term, len(args))
	for idx, arg := range args {
		converted, err := p.host.toPolar(arg)
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
	ffiQuery, err := p.ffiPolar.newQueryFromTerm(Term{Value{inner}})
	if err != nil {
		return nil, err
	}
	newQuery := newQuery(*ffiQuery, p.host.copy())
	return &newQuery, nil
}

func (p Polar) Repl(files ...string) error {
	return fmt.Errorf("Go REPL is not yet implemented")
}

//     def repl(self, files=[]):
//         """Start an interactive REPL session."""
//         for f in files:
//             self.load_file(f)

//         while True:
//             try:
//                 query = input(FG_BLUE + "query> " + RESET).strip(";")
//             except (EOFError, KeyboardInterrupt):
//                 return
//             try:
//                 ffi_query = self.ffi_polar.new_query_from_str(query)
//             except ParserError as e:
//                 print_error(e)
//                 continue

//             result = False
//             try:
//                 query = Query(ffi_query, host=self.host.copy()).run()
//                 for res in query:
//                     result = True
//                     bindings = res["bindings"]
//                     if bindings:
//                         for variable, value in bindings.items():
//                             print(variable + " = " + repr(value))
//                     else:
//                         print(True)
//             except PolarRuntimeError as e:
//                 print_error(e)
//                 continue
//             if not result:
//                 print(False)

func (p Polar) RegisterClass(cls reflect.Type, name *string) error {
	var className string
	if name == nil {
		className = cls.Name()
	} else {
		className = *name
	}

	err := p.host.cacheClass(cls, className)
	if err != nil {
		return err
	}
	// zeroVal := reflect.Zero(cls)
	newVal := reflect.New(cls)
	return p.RegisterConstant(newVal.Interface(), className)
}

func (p Polar) RegisterConstant(value interface{}, name string) error {
	polarValue, err := p.host.toPolar(value)
	if err != nil {
		return err
	}
	return p.ffiPolar.registerConstant(Term{*polarValue}, name)
}
