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

func NewPolar() Polar {
	ffiPolar := NewPolarFfi()
	polar := Polar{
		ffiPolar: ffiPolar,
		host:     NewHost(ffiPolar),
	}

	// register global constants
	return polar
}

// CLASSES: Dict[str, type] = {}

// class Polar:
//     """Polar API"""

//     def __init__(self, classes=CLASSES):
//         self.ffi_polar = FfiPolar()
//         self.host = Host(self.ffi_polar)

//         # Register global constants.
//         self.register_constant(None, name="nil")

//         # Register built-in classes.
//         self.register_class(bool, name="Boolean")
//         self.register_class(int, name="Integer")
//         self.register_class(float, name="Float")
//         self.register_class(list, name="List")
//         self.register_class(dict, name="Dictionary")
//         self.register_class(str, name="String")
//         self.register_class(datetime, name="Datetime")
//         self.register_class(timedelta, name="Timedelta")

//         # Pre-registered classes.
//         for name, cls in classes.items():
//             self.register_class(cls, name=name)

//     def __del__(self):
//         del self.host
//         del self.ffi_polar

func (p Polar) checkInlineQueries() error {
	for {
		query, err := p.ffiPolar.nextInlineQuery()
		if err != nil {
			return err
		}
		if query == nil {
			return nil
		}
		// TODO
		// try:
		// 	next(Query(query, host=self.host.copy()).run())
		// except StopIteration:
		// 	source = query.source()
		// 	raise InlineQueryFailedError(source.get())
	}

	return nil
}

func (p Polar) LoadFile(f string) error {
	if filepath.Ext(f) != "polar" {
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

// Query accepts string and predicates
func (p Polar) Query(query interface{}) (*Query, error) {
	switch query.(type) {
	case string:
		ffiQuery, err := p.ffiPolar.newQueryFromStr(query.(string))
		if err != nil {
			return nil, err
		}
		newQuery := newQuery(*ffiQuery, p.host.copy())
		return &newQuery, nil
	}

	return nil, fmt.Errorf("Unsupported query type: %v", query)
}

func (p Polar) QueryRule(name string, args ...interface{}) (*Query, error) {
	// TODO
	return nil, fmt.Errorf("Unsupported: QueryRule")
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
	err := p.host.cacheClass(cls, name)
	if err != nil {
		return err
	}
	return p.RegisterConstant(cls, cls.Name())
}

func (p Polar) RegisterConstant(value interface{}, name string) error {
	polarValue, err := p.host.toPolar(value)
	if err != nil {
		return err
	}
	return p.ffiPolar.registerConstant(*polarValue, name)
}
