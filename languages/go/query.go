package oso

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"reflect"

	"github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/interfaces"
	"github.com/osohq/go-oso/internal/ffi"
	"github.com/osohq/go-oso/internal/host"
	"github.com/osohq/go-oso/internal/util"
	"github.com/osohq/go-oso/types"
	. "github.com/osohq/go-oso/types"
)

/*
Execute a Polar query through the FFI/event interface.
*/
type Query struct {
	ffiQuery ffi.QueryFfi
	host     host.Host
	calls    map[uint64]<-chan interface{}
}

// NATIVE_TYPES = [int, float, bool, str, dict, type(None), list]

func newQuery(ffiQuery ffi.QueryFfi, host host.Host) Query {
	return Query{
		ffiQuery: ffiQuery,
		host:     host,
		calls:    make(map[uint64]<-chan interface{}),
	}
}

func (q *Query) Cleanup() {
	q.ffiQuery.Delete()
}

func (q *Query) resultsChannel() (<-chan map[string]interface{}, <-chan error) {
	results := make(chan map[string]interface{}, 1)
	errors := make(chan error, 1)

	go func() {
		r, err := q.Next()
		for r != nil && err == nil {
			results <- *r
			r, err = q.Next()
		}
		if err != nil {
			errors <- err
		}
		close(results)
		close(errors)
	}()

	return results, errors
}

/*
Executes the query until all results have been returned, and returns results
as a list of binding maps.
*/
func (q *Query) GetAllResults() ([]map[string]interface{}, error) {
	results := make([]map[string]interface{}, 0)
	for {
		if v, err := q.Next(); err != nil {
			return nil, err
		} else if v == nil {
			break
		} else {
			results = append(results, *v)
		}
	}
	return results, nil
}

/*
Get the next query result. Returns a pointer to a map of result bindings,
or a nil pointer if there are no results.
*/
func (q *Query) Next() (*map[string]interface{}, error) {
	if q == nil {
		return nil, fmt.Errorf("query has already finished")
	}
	for {
		ffiEvent, err := q.ffiQuery.NextEvent()
		if err != nil {
			return nil, err
		}
		var event QueryEvent
		err = json.Unmarshal([]byte(*ffiEvent), &event)
		if err != nil {
			return nil, err
		}

		switch ev := event.QueryEventVariant.(type) {
		case QueryEventDone:
			defer q.Cleanup()
			return nil, nil
		case QueryEventDebug:
			err = q.handleDebug(ev)
		case QueryEventResult:
			results := make(map[string]interface{})
			for k, v := range ev.Bindings {
				converted, err := q.host.ToGo(v)
				if err != nil {
					return nil, err
				}
				results[string(k)] = converted
			}
			return &results, nil
		case QueryEventMakeExternal:
			err = q.handleMakeExternal(ev)
		case QueryEventExternalCall:
			err = q.handleExternalCall(ev)
		case QueryEventExternalIsa:
			err = q.handleExternalIsa(ev)
		case QueryEventExternalIsaWithPath:
			err = q.handleExternalIsaWithPath(ev)
		case QueryEventExternalIsSubSpecializer:
			err = q.handleExternalIsSubSpecializer(ev)
		case QueryEventExternalIsSubclass:
			err = q.handleExternalIsSubclass(ev)
		case QueryEventExternalOp:
			err = q.handleExternalOp(ev)
		case QueryEventNextExternal:
			err = q.handleNextExternal(ev)
		default:
			defer q.Cleanup()
			return nil, fmt.Errorf("unexpected query event: %v", ev)
		}
		if err != nil {
			defer q.Cleanup()
			return nil, err
		}
	}

}

func (q *Query) Bind(name string, value *types.Term) error {
	return q.ffiQuery.Bind(name, value)
}

/*
Set whether the Host accepts Expression types from Polar, or raises an error.
*/
func (q *Query) SetAcceptExpression(acceptExpression bool) {
	(q.host).SetAcceptExpression(acceptExpression)
}

func (q Query) handleMakeExternal(event types.QueryEventMakeExternal) error {
	id := uint64(event.InstanceId)
	call, _ := event.Constructor.Value.ValueVariant.(ValueCall)
	if call.Kwargs != nil {
		return &errors.KwargsError{}
	}
	return q.host.MakeInstance(call, id)
}

func (q Query) handleRelation(event types.QueryEventExternalCall, instance interface{}, attr string, rel Relation) error {
	// otherwise look up the field
	value := reflect.ValueOf(instance).FieldByName(rel.MyField)
	if !value.IsValid() {
		q.ffiQuery.ApplicationError((errors.NewMissingAttributeError(instance, attr)).Error())
		q.ffiQuery.CallResult(event.CallId, nil)
		return nil
	}

	cond := FilterCondition{
		Datum{Projection{rel.OtherType, rel.OtherField}},
		Eq,
		Datum{Immediate{value.Interface()}},
	}

	filter := Filter{
		rel.OtherType,
		[]FilterRelation{},
		[][]FilterCondition{{cond}},
		q.host.GetFields(),
	}

	adapter := *q.host.GetAdapter()
	query, err := adapter.BuildQuery(&filter)
	if err != nil {
		return err
	}
	res, err := adapter.ExecuteQuery(query)
	if err != nil {
		return err
	}

	if rel.Kind == "one" {
		if len(res) != 1 {
			return fmt.Errorf("Expected one result, got %d", len(res))
		}
		polarValue, err := q.host.ToPolar(res[0])
		if err != nil {
			return err
		}
		return q.ffiQuery.CallResult(event.CallId, &Term{*polarValue})
	} else {
		polarValue, err := q.host.ToPolar(res)
		if err != nil {
			return err
		}
		return q.ffiQuery.CallResult(event.CallId, &Term{*polarValue})
	}
}

func (q Query) handleExternalCall(event types.QueryEventExternalCall) error {
	instance, err := q.host.ToGo(event.Instance)
	if err != nil {
		return err
	}

	attr := string(event.Attribute)

	var result interface{}

	// if we provided Args, it should be callable
	if event.Args != nil {
		// Check for the method on a pointer to the value, not the value itself.
		typ := reflect.TypeOf(instance)
		iv := reflect.New(typ)
		iv.Elem().Set(reflect.ValueOf(instance))
		method := iv.MethodByName(attr)

		if !method.IsValid() {
			q.ffiQuery.ApplicationError((errors.NewMissingAttributeError(instance, string(event.Attribute))).Error())
			q.ffiQuery.CallResult(event.CallId, nil)
			return nil
		}
		if method.Kind() == reflect.Func {
			results, err := q.host.CallFunction(method, *event.Args)
			if err != nil {
				return &errors.ErrorWithAdditionalInfo{Inner: errors.NewInvalidCallError(instance, string(event.Attribute)), Info: err.Error()}
			}

			// maybe: This is kind of odd, maybe error instead if len(results) > 1
			// Right now if you called a function that returns an error you'll get back
			// a list [result, nil] or something.
			// It does work the same way in python if you return a tuple though so maybe it's fine.
			// You could destructure it in polar if you want.
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
			return errors.NewInvalidCallError(instance, attr)
		}
	} else {
		// this might be a relation
		rel, err := q.host.GetRelation(instance, attr)
		if err != nil {
			return err
		}
		if rel != nil {
			return q.handleRelation(event, instance, attr, *rel)
		}

		// otherwise look up the field
		value := reflect.ValueOf(instance).FieldByName(attr)
		if !value.IsValid() {
			q.ffiQuery.ApplicationError((errors.NewMissingAttributeError(instance, attr)).Error())
			q.ffiQuery.CallResult(event.CallId, nil)
			return nil
		}
		result = value.Interface()
	}

	polarValue, err := q.host.ToPolar(result)
	if err != nil {
		return err
	}
	return q.ffiQuery.CallResult(event.CallId, &Term{*polarValue})
}

func (q Query) handleExternalIsa(event types.QueryEventExternalIsa) error {
	isa, err := q.host.Isa(event.Instance, string(event.ClassTag))
	if err != nil {
		return err
	}
	return q.ffiQuery.QuestionResult(event.CallId, isa)
}

func (q Query) handleExternalIsaWithPath(event types.QueryEventExternalIsaWithPath) error {
	sup := string(event.ClassTag)
	bas := string(event.BaseTag)
	path := []interface{}{}
	for _, term := range event.Path {
		val, err := q.host.ToGo(term)
		if err != nil {
			return err
		}
		path = append(path, val)
	}
	sub := bas
	for _, item := range path {
		switch it := q.host.GetField(sub, item.(string)).(type) {
		case types.Relation:
			sub = it.OtherType
		case string:
			sub = it
		}
	}
	answer, err := q.host.IsSubclass(sub, sup)
	if err != nil {
		return err
	}
	return q.ffiQuery.QuestionResult(event.CallId, answer)
}

func (q Query) handleExternalIsSubSpecializer(event types.QueryEventExternalIsSubSpecializer) error {
	res, err := q.host.IsSubspecializer(int(event.InstanceId), string(event.LeftClassTag), string(event.RightClassTag))
	if err != nil {
		return err
	}
	return q.ffiQuery.QuestionResult(event.CallId, res)
}

func (q Query) handleExternalIsSubclass(event types.QueryEventExternalIsSubclass) error {
	res, err := q.host.IsSubclass(string(event.LeftClassTag), string(event.RightClassTag))
	if err != nil {
		return err
	}
	return q.ffiQuery.QuestionResult(event.CallId, res)
}

func (q Query) handleExternalOp(event types.QueryEventExternalOp) error {
	if len(event.Args) != 2 {
		return fmt.Errorf("Unexpected number of arguments for operation: %v", len(event.Args))
	}
	left, err := q.host.ToGo(event.Args[0])
	if err != nil {
		return err
	}
	right, err := q.host.ToGo(event.Args[1])
	if err != nil {
		return err
	}

	leftCmp, leftOk := left.(interfaces.Comparer)
	rightCmp, rightOk := right.(interfaces.Comparer)
	op := event.Operator.OperatorVariant

	// this logic is kind of weird!
	// the reason why we need so many different comparison
	// routines is that interfaces.Comparer only has methods
	// to test for == and <, and x > y = !(x < y || x == y)
	// is only true if x and y are actually ordered -- which
	// we can't assume. for that reason different subsets of
	// the 6 usual comparison operators are available in each
	// case where x or y implements or doesn't implement
	// interfaces.Comparer.

	if leftOk {
		if rightOk {
			return q.handleCmpLR(event, leftCmp, op, rightCmp)
		}
		return q.handleCmpL(event, leftCmp, op, right)
	}
	if rightOk {
		return q.handleCmpR(event, left, op, rightCmp)
	}
	return q.handleCmp(event, left, op, right)
}

func (q Query) answer(ev types.QueryEventExternalOp, b bool) error {
	return q.ffiQuery.QuestionResult(ev.CallId, b)
}

func (q Query) handleCmpL(
	ev types.QueryEventExternalOp,
	l interfaces.Comparer,
	op OperatorVariant,
	r interface{}) error {

	switch op.(type) {
	case OperatorLt:
		return q.answer(ev, l.Lt(r))
	case OperatorLeq:
		return q.answer(ev, l.Lt(r) || l.Equal(r))
	case OperatorEq:
		return q.answer(ev, l.Equal(r))
	case OperatorNeq:
		return q.answer(ev, !l.Equal(r))
	default:
		return fmt.Errorf("Unsupported operation: %v", op)
	}
}

func (q Query) handleCmpR(
	ev types.QueryEventExternalOp,
	l interface{},
	op OperatorVariant,
	r interfaces.Comparer) error {

	switch op.(type) {
	case OperatorGt:
		return q.answer(ev, r.Lt(l))
	case OperatorGeq:
		return q.answer(ev, r.Lt(l) || r.Equal(l))
	case OperatorEq:
		return q.answer(ev, r.Equal(l))
	case OperatorNeq:
		return q.answer(ev, !r.Equal(l))
	default:
		return fmt.Errorf("Unsupported operation: %v", op)
	}
}

func (q Query) handleCmpLR(
	ev types.QueryEventExternalOp,
	l interfaces.Comparer,
	op OperatorVariant,
	r interfaces.Comparer) error {

	switch op.(type) {
	case OperatorLt:
		return q.answer(ev, l.Lt(r))
	case OperatorLeq:
		return q.answer(ev, l.Lt(r) || l.Equal(r))
	case OperatorGt:
		return q.answer(ev, r.Lt(l))
	case OperatorGeq:
		return q.answer(ev, r.Lt(l) || r.Equal(l))
	case OperatorEq:
		return q.answer(ev, l.Equal(r))
	case OperatorNeq:
		return q.answer(ev, !l.Equal(r))
	default:
		return fmt.Errorf("Unsupported operation: %v", op)
	}
}

func (q Query) handleCmp(
	ev types.QueryEventExternalOp,
	l interface{},
	op OperatorVariant,
	r interface{}) error {

	switch op.(type) {
	case OperatorEq:
		return q.answer(ev, reflect.DeepEqual(l, r))
	case OperatorNeq:
		return q.answer(ev, !reflect.DeepEqual(l, r))
	default:
		return fmt.Errorf("Unsupported operation: %v", op)
	}
}

func (q Query) handleNextExternal(event types.QueryEventNextExternal) error {
	if _, ok := q.calls[event.CallId]; !ok {
		instance, err := q.host.ToGo(event.Iterable)
		if err != nil {
			return err
		}
		if iter, ok := instance.(interfaces.Iterator); ok {
			q.calls[event.CallId] = iter.Iter()
		} else {
			return errors.NewInvalidIteratorError(instance)
		}
	}

	iter := q.calls[event.CallId]
	nextValue, ok := <-iter
	if !ok { // iterator is done
		return q.ffiQuery.CallResult(event.CallId, nil)
	}
	retValue, err := q.host.ToPolar(nextValue)
	if err != nil {
		return err
	}
	return q.ffiQuery.CallResult(event.CallId, &Term{*retValue})
}

func (q Query) handleDebug(event types.QueryEventDebug) error {
	fmt.Printf("%s\n", event.Message)

	reader := bufio.NewReader(os.Stdin)
	fmt.Print("debug> ")
	text, _ := reader.ReadString('\n')
	text = util.QueryStrip(text)

	if text == "" {
		text = "continue"
	}

	err := q.ffiQuery.DebugCommand(&text)
	return err
}
