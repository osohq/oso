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
			break
		case *QueryEventExternalCall:
			err = q.handleExternalCall(ev)
			break
		case *QueryEventExternalIsa:
			err = q.handleExternalIsa(ev)
			break
		case *QueryEventExternalIsSubSpecializer:
			err = q.handleExternalIsSubSpecializer(ev)
			break
		case *QueryEventExternalIsSubclass:
			err = q.handleExternalIsSubclass(ev)
			break
		case *QueryEventExternalUnify:
			err = q.handleExternalUnify(ev)
			break
		case *QueryEventExternalOp:
			err = q.handleExternalOp(ev)
			break
		case *QueryEventNextExternal:
			err = q.handleNextExternal(ev)
			break
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
	return fmt.Errorf("handleExternalIsSubSpecializer not yet implemented")
}
func (q Query) handleExternalIsSubclass(event *QueryEventExternalIsSubclass) error {
	return fmt.Errorf("handleExternalIsSubclass not yet implemented")
}
func (q Query) handleExternalUnify(event *QueryEventExternalUnify) error {
	return fmt.Errorf("handleExternalUnify not yet implemented")
}
func (q Query) handleExternalOp(event *QueryEventExternalOp) error {
	return fmt.Errorf("handleExternalOp not yet implemented")
}
func (q Query) handleNextExternal(event *QueryEventNextExternal) error {
	return fmt.Errorf("handleNextExternal not yet implemented")
}

//             if kind == "Done":
//                 break
//             elif kind == "Result":
//                 bindings = {
//                     k: self.host.to_python(v) for k, v in data["bindings"].items()
//                 }
//                 trace = data["trace"]
//                 yield {"bindings": bindings, "trace": trace}
//             elif kind in call_map:
//                 call_map[kind](data)
//             else:
//                 raise PolarRuntimeError(f"Unhandled event: {json.dumps(event)}")

//     def handle_make_external(self, data):
//         id = data["instance_id"]
//         constructor = data["constructor"]["value"]
//         if "Call" in constructor:
//             cls_name = constructor["Call"]["name"]
//             args = [self.host.to_python(arg) for arg in constructor["Call"]["args"]]
//             kwargs = constructor["Call"]["kwargs"] or {}
//             kwargs = {k: self.host.to_python(v) for k, v in kwargs.items()}
//         else:
//             raise InvalidConstructorError()
//         self.host.make_instance(cls_name, args, kwargs, id)

//     def handle_external_call(self, data):
//         call_id = data["call_id"]
//         instance = self.host.to_python(data["instance"])

//         attribute = data["attribute"]

//         # Lookup the attribute on the instance.
//         try:
//             attr = getattr(instance, attribute)
//         except AttributeError as e:
//             self.ffi_query.application_error(str(e))
//             self.ffi_query.call_result(call_id, None)
//             return
//         if (
//             callable(attr) and not data["args"] is None
//         ):  # If it's a function, call it with the args.
//             args = [self.host.to_python(arg) for arg in data["args"]]
//             kwargs = data["kwargs"] or {}
//             kwargs = {k: self.host.to_python(v) for k, v in kwargs.items()}
//             result = attr(*args, **kwargs)
//         elif not data["args"] is None:
//             raise InvalidCallError(
//                 f"tried to call '{attribute}' but it is not callable"
//             )
//         else:  # If it's just an attribute, it's the result.
//             result = attr

//         # Return the result of the call.
//         self.ffi_query.call_result(call_id, self.host.to_polar(result))

//     def handle_external_op(self, data):
//         op = data["operator"]
//         args = [self.host.to_python(arg) for arg in data["args"]]
//         answer = self.host.operator(op, args)
//         self.ffi_query.question_result(data["call_id"], answer)

//     def handle_external_isa(self, data):
//         instance = data["instance"]
//         class_tag = data["class_tag"]
//         answer = self.host.isa(instance, class_tag)
//         self.ffi_query.question_result(data["call_id"], answer)

//     def handle_external_unify(self, data):
//         left_instance_id = data["left_instance_id"]
//         right_instance_id = data["right_instance_id"]
//         answer = self.host.unify(left_instance_id, right_instance_id)
//         self.ffi_query.question_result(data["call_id"], answer)

//     def handle_external_is_subspecializer(self, data):
//         instance_id = data["instance_id"]
//         left_tag = data["left_class_tag"]
//         right_tag = data["right_class_tag"]
//         answer = self.host.is_subspecializer(instance_id, left_tag, right_tag)
//         self.ffi_query.question_result(data["call_id"], answer)

//     def handle_external_is_subclass(self, data):
//         left_tag = data["left_class_tag"]
//         right_tag = data["right_class_tag"]
//         answer = self.host.is_subclass(left_tag, right_tag)
//         self.ffi_query.question_result(data["call_id"], answer)

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
