package oso

import (
	"encoding/json"
	"fmt"
	"reflect"
)

type Query struct {
	ffiQuery QueryFfi
	host     Host
	calls    map[int]interface{}
}

// NATIVE_TYPES = [int, float, bool, str, dict, type(None), list]

func newQuery(ffiQuery QueryFfi, host Host) Query {
	return Query{
		ffiQuery: ffiQuery,
		host:     host,
		calls:    make(map[int]interface{}),
	}
}

func (q *Query) Next() (*map[string]interface{}, error) {
	if q == nil {
		return nil, fmt.Errorf("query has already finished")
	}
	for {
		ffiEvent, err := q.ffiQuery.nextEvent()
		if err != nil {
			return nil, err
		}
		var event QueryEvent
		err = json.Unmarshal([]byte(*ffiEvent), &event)
		if err != nil {
			return nil, err
		}

		switch ev := event.QueryEventVariant.(type) {
		case *QueryEventNone:
			// nothing to do
			continue
		case *QueryEventDone:
			defer q.ffiQuery.delete()
			return nil, nil
		case *QueryEventDebug:
			// TODO
			return nil, fmt.Errorf("not yet implemented")
		case *QueryEventResult:
			results := make(map[string]interface{})
			for k, v := range ev.Bindings {
				converted, err := q.host.toGo(v)
				if err != nil {
					return nil, err
				}
				results[k] = converted
			}
			return &results, nil
		case *QueryEventMakeExternal:
			err = q.handleMakeExternal(ev)
		case *QueryEventExternalCall:
			err = q.handleExternalCall(ev)
		case *QueryEventExternalIsa:
			err = q.handleExternalIsa(ev)
		case *QueryEventExternalIsSubSpecializer:
			err = q.handleExternalIsSubSpecializer(ev)
		case *QueryEventExternalIsSubclass:
			err = q.handleExternalIsSubclass(ev)
		case *QueryEventExternalUnify:
			err = q.handleExternalUnify(ev)
		case *QueryEventExternalOp:
			err = q.handleExternalOp(ev)
		case *QueryEventNextExternal:
			err = q.handleNextExternal(ev)
		default:
			return nil, fmt.Errorf("unexpected query event: %v", ev)
		}
		if err != nil {
			return nil, err
		}
	}

}

func (q Query) handleMakeExternal(event *QueryEventMakeExternal) error {
	if ctor, ok := event.Constructor.ValueVariant.(*ValueCall); ok {
		args := make([]interface{}, len(ctor.Args))
		for idx, arg := range ctor.Args {
			converted, err := q.host.toGo(arg)
			if err != nil {
				return err
			}
			args[idx] = converted
		}
		kwargs := make(map[string]interface{})
		if ctor.Kwargs != nil {
			for k, v := range *ctor.Kwargs {
				converted, err := q.host.toGo(v)
				if err != nil {
					return err
				}
				kwargs[k] = converted
			}
		}
		_, err := q.host.makeInstance(ctor.Name, args, kwargs, int(event.InstanceId))
		return err
	}
	return &InvalidConstructorError{ctor: event.Constructor}
}

func (q Query) handleExternalCall(event *QueryEventExternalCall) error {
	instance, err := q.host.toGo(event.Instance)
	if err != nil {
		return err
	}

	var result interface{}

	// if we provided Args, it should be callable
	if event.Args != nil {
		method := reflect.ValueOf(instance).MethodByName(event.Attribute)
		if !method.IsValid() {
			q.ffiQuery.applicationError((&InvalidCallError{instance: event.Instance, field: event.Attribute}).Error())
			q.ffiQuery.callResult(int(event.CallId), nil)
			return nil
		}
		if method.Kind() == reflect.Func {
			args, err := q.host.listToGo(*event.Args)
			valueArgs := make([]reflect.Value, len(args))
			for idx, v := range args {
				valueArgs[idx] = reflect.ValueOf(v)
			}
			if err != nil {
				return err
			}
			if event.Kwargs != nil {
				return &KwargsError{}
			}
			results := method.Call(valueArgs)
			if len(results) == 1 {
				result = results[0].Interface()
			} else {
				arrayResult := make([]interface{}, len(results))
				for idx, v := range results {
					arrayResult[idx] = v.Interface()
				}
				result = interface{}(arrayResult)
			}
		} else {
			return &InvalidCallError{instance: event.Instance, field: event.Attribute}
		}
	} else {
		attr := reflect.ValueOf(instance).FieldByName(event.Attribute)
		if !attr.IsValid() {
			q.ffiQuery.applicationError((&InvalidCallError{instance: event.Instance, field: event.Attribute}).Error())
			q.ffiQuery.callResult(int(event.CallId), nil)
			return nil
		}
		result = attr.Interface()
	}

	polarValue, err := q.host.toPolar(result)
	if err != nil {
		return err
	}
	return q.ffiQuery.callResult(int(event.CallId), polarValue)
}
func (q Query) handleExternalIsa(event *QueryEventExternalIsa) error {
	isa, err := q.host.isa(event.Instance, event.ClassTag)
	if err != nil {
		return err
	}
	return q.ffiQuery.questionResult(int(event.CallId), isa)
}

func (q Query) handleExternalIsSubSpecializer(event *QueryEventExternalIsSubSpecializer) error {
	res, err := q.host.isSubspecializer(int(event.InstanceId), event.LeftClassTag, event.RightClassTag)
	if err != nil {
		return err
	}
	return q.ffiQuery.questionResult(int(event.CallId), res)
}

func (q Query) handleExternalIsSubclass(event *QueryEventExternalIsSubclass) error {
	res, err := q.host.isSubclass(event.LeftClassTag, event.RightClassTag)
	if err != nil {
		return err
	}
	return q.ffiQuery.questionResult(int(event.CallId), res)
}

func (q Query) handleExternalUnify(event *QueryEventExternalUnify) error {
	res, err := q.host.unify(int(event.LeftInstanceId), int(event.RightInstanceId))
	if err != nil {
		return err
	}
	return q.ffiQuery.questionResult(int(event.CallId), res)
}

func (q Query) handleExternalOp(event *QueryEventExternalOp) error {
	if len(event.Args) != 2 {
		return fmt.Errorf("Unexpected number of arguments for operation: %v", len(event.Args))
	}
	left, err := q.host.toGo(event.Args[0])
	if err != nil {
		return err
	}
	right, err := q.host.toGo(event.Args[1])
	if err != nil {
		return err
	}
	var answer bool
	leftCmp := left.(Comparer)
	rightCmp := right.(Comparer)
	switch event.Operator.OperatorVariant.(type) {
	case *OperatorLt:
		answer = leftCmp.Lt(rightCmp)
	case *OperatorLeq:
		answer = leftCmp.Lt(rightCmp) || leftCmp.Equal(rightCmp)
	case *OperatorGt:
		answer = rightCmp.Lt(leftCmp)
	case *OperatorGeq:
		answer = !leftCmp.Lt(rightCmp)
	case *OperatorEq:
		answer = leftCmp.Equal(rightCmp)
	case *OperatorNeq:
		answer = !leftCmp.Equal(rightCmp)
	default:
		return fmt.Errorf("Unsupported operation: %v", event.Operator.OperatorVariant)
	}
	return q.ffiQuery.questionResult(int(event.CallId), answer)
}

func (q Query) handleNextExternal(event *QueryEventNextExternal) error {
	return fmt.Errorf("handleNextExternal not yet implemented")
}

//     def handle_next_external(self, data):
//         call_id = data["call_id"]
//         iterable = data["iterable"]

//         if call_id not in self.calls:
//             value = self.host.to_python(iterable)
//             if isinstance(value, Iterable):
//                 self.calls[call_id] = iter(value)
//             else:
//                 raise InvalidIteratorError(f"{value} is not iterable")

//         # Return the next result of the call.
//         try:
//             value = next(self.calls[call_id])
//             self.ffi_query.call_result(call_id, self.host.to_polar(value))
//         except StopIteration:
//             self.ffi_query.call_result(call_id, None)

//     def handle_debug(self, data):
//         if data["message"]:
//             print(data["message"])
//         try:
//             command = input("debug> ").strip(";")
//         except EOFError:
//             command = "continue"
//         self.ffi_query.debug_command(self.host.to_polar(command))
