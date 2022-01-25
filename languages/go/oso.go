package oso

import (
	"errors"
	"fmt"
	"os"
	"runtime"

	osoErrors "github.com/osohq/go-oso/errors"
	"github.com/osohq/go-oso/internal/host"
	"github.com/osohq/go-oso/types"
)

/*
The central object to manage policy state and verify requests.
*/
type Oso struct {
	p              *Polar
	readAction     interface{}
	forbiddenError func() error
	notFoundError  func() error
}

/*
Construct a new Oso instance.

	import oso "github.com/osohq/go-oso"
	if o, err := oso.NewOso(); err != nil {
		t.Fatalf("Failed to set up Oso: %v", err)
	}
*/
func NewOso() (Oso, error) {
	if p, e := newPolar(); e != nil {
		return Oso{}, e
	} else {
		return Oso{
			p:              p,
			readAction:     "read",
			forbiddenError: func() error { return &osoErrors.ForbiddenError{} },
			notFoundError:  func() error { return &osoErrors.NotFoundError{} },
		}, nil
	}
}

func (o *Oso) GetHost() *host.Host {
	return &o.p.host
}

/*
Override the "read" action, which is used to differentiate between a
NotFoundError and a ForbiddenError on authorization failures.

	o, _ = oso.NewOso()
	o.SetReadAction("READ")
*/
func (o *Oso) SetReadAction(readAction interface{}) {
	o.readAction = readAction
}

/*
Override the default ForbiddenError, returned when authorization fails.

	o, _ = oso.NewOso()
	o.SetForbiddenError(func() error { return &MyCustomError{} })
*/
func (o *Oso) SetForbiddenError(forbiddenError func() error) {
	o.forbiddenError = forbiddenError
}

/*
Override the default NotFoundError, returned by the Authorize method when a user
does not have read permission.

	o, _ = oso.NewOso()
	o.SetNotFoundError(func() error { return &MyCustomError{} })
*/
func (o *Oso) SetNotFoundError(notFoundError func() error) {
	o.notFoundError = notFoundError
}

/*
Load Polar policy from ".polar" files, checking that all inline queries succeed.
*/
func (o Oso) LoadFiles(files []string) error {
	return (*o.p).loadFiles(files)
}

/*
Load Polar policy from a ".polar" file, checking that all inline queries succeed.

Deprecated: `Oso.LoadFile` has been deprecated in favor of `Oso.LoadFiles` as
of the 0.20 release. Please see changelog for migration instructions:
https://docs.osohq.com/project/changelogs/2021-09-15.html
*/
func (o Oso) LoadFile(f string) error {
	fmt.Fprintln(os.Stderr,
		"`Oso.LoadFile` has been deprecated in favor of `Oso.LoadFiles` as of the 0.20 release.\n\n"+
			"Please see changelog for migration instructions: https://docs.osohq.com/project/changelogs/2021-09-15.html")
	return (*o.p).loadFiles([]string{f})
}

/*
Load Polar policy from a string, checking that all inline queries succeed.
*/
func (o Oso) LoadString(s string) error {
	return (*o.p).loadString(s)
}

/*
Clear all rules from the Oso knowledge base (i.e., remove all loaded policies).
*/
func (o Oso) ClearRules() error {
	return (*o.p).clearRules()
}

/*
Register a Go type so that it can be referenced within Polar files. Accepts a
concrete value of the Go type and a constructor function or nil if no
constructor is required.
*/
func (o Oso) RegisterClass(cls interface{}, ctor interface{}) error {
	return (*o.p).registerClass(cls, ctor, nil, nil)
}

/*
Register a Go type under a certain name/alias so that it can be referenced
within Polar files by that name. Accepts a concrete value of the Go type and a
constructor function or nil if no constructor is required.
*/
func (o Oso) RegisterClassWithName(cls interface{}, ctor interface{}, name string) error {
	return (*o.p).registerClass(cls, ctor, &name, nil)
}

/*
Register a Go type under a certain name/alias so that it can be referenced
within Polar files by that name. Accepts a concrete value of the Go type and a
constructor function or nil if no constructor is required.
*/
func (o Oso) RegisterClassWithNameAndFields(cls interface{}, ctor interface{}, name string, fields map[string]interface{}) error {
	return (*o.p).registerClass(cls, ctor, &name, fields)
}

/*
Register a Go value as a Polar constant variable called `name`.
*/
func (o Oso) RegisterConstant(value interface{}, name string) error {
	return (*o.p).registerConstant(value, name)
}

/*
Query the policy using a query string; the query is run in a new Go routine.
Accepts the string to query for.
Returns a channel of resulting binding maps, and a channel for errors.
As the query is evaluated, all resulting bindings will be written to the results channel,
and any errors will be written to the error channel.
The results channel must be completely consumed or it will leak memory.
*/
func (o Oso) QueryStr(q string) (<-chan map[string]interface{}, <-chan error) {
	if query, err := (*o.p).queryStr(q); err != nil {
		errors := make(chan error, 1)
		go func() {
			errors <- err
			close(errors)
		}()
		return nil, errors
	} else {
		return query.resultsChannel()
	}
}

/*
Query the policy for a rule; the query is run in a new Go routine.
Accepts the name of the rule to query, and a variadic list of rule arguments.
Returns a channel of resulting binding maps, and a channel for errors.
As the query is evaluated, all resulting bindings will be written to the results channel,
and any errors will be written to the error channel.
The results channel must be completely consumed or it will leak memory.
*/
func (o Oso) QueryRule(name string, args ...interface{}) (<-chan map[string]interface{}, <-chan error) {
	if query, err := (*o.p).queryRule(name, args...); err != nil {
		errors := make(chan error, 1)
		go func() {
			errors <- err
			close(errors)
		}()
		return nil, errors
	} else {
		return query.resultsChannel()
	}
}

/*
Query the policy for a rule, and return true if there are any results. Returns
false if there are no results.
*/
func (o Oso) QueryRuleOnce(name string, args ...interface{}) (bool, error) {
	query, err := (*o.p).queryRule(name, args...)
	if err != nil {
		return false, err
	}
	results, err := query.Next()
	if err != nil {
		return false, err
	} else if results != nil {
		// Manually clean up query since we are not pulling all results.
		defer query.Cleanup()
		return true, nil
	} else {
		return false, nil
	}
}

/*
Create policy query from a query string.
Accepts the string to query for.
Returns a new *Query, on which `Next()` can be called to get the next result,
or an error.
*/
func (o Oso) NewQueryFromStr(q string) (*Query, error) {
	return (*o.p).queryStr(q)
}

/*
Create policy query for a rule.
Accepts the name of the rule to query, and a variadic list of rule arguments.
Returns a new *Query, on which `Next()` can be called to get the next result,
or an error.
*/
func (o Oso) NewQueryFromRule(name string, args ...interface{}) (*Query, error) {
	return (*o.p).queryRule(name, args...)
}

/*
Check if an (actor, action, resource) combination is allowed by the policy.
Returns the result as a bool, or an error.
*/
func (o Oso) IsAllowed(actor interface{}, action interface{}, resource interface{}) (bool, error) {
	return o.QueryRuleOnce("allow", actor, action, resource)
}

/*
Return a set of actions allowed by the given (actor, resource) combination allowed
by the policy.

Deprecated: Use AuthorizedActions instead.
*/
func (o Oso) GetAllowedActions(actor interface{}, resource interface{}, allowWildcard bool) (map[interface{}]struct{}, error) {
	return o.AuthorizedActions(actor, resource, allowWildcard)
}

/*
Ensure that `actor` is allowed to perform `action` on `resource`.

If the action is permitted with an `allow` rule in the policy, then this method
returns `nil`. If the action is not permitted by the policy, this method will
return an error.

The error returned by this method depends on whether the actor can perform the
`"read"` action on the resource. If they cannot read the resource, then a
`NotFoundError` error is returned. Otherwise, a `ForbiddenError` is returned.

You can customize the errors returned by this function using the
`SetReadAction`, `SetForbiddenError`, and `SetNotFoundError` configuration
functions.
*/
func (o Oso) Authorize(actor interface{}, action interface{}, resource interface{}) error {
	isAllowed, err := o.QueryRuleOnce("allow", actor, action, resource)
	if err != nil {
		return err
	}

	if isAllowed {
		return nil
	}

	// Decide whether to return not found or forbidden error
	isNotFound := false
	if action == o.readAction {
		isNotFound = true
	} else {
		isReadAllowed, err := o.QueryRuleOnce("allow", actor, o.readAction, resource)
		if err != nil {
			return err
		}
		if !isReadAllowed {
			isNotFound = true
		}
	}

	if isNotFound {
		return o.notFoundError()
	} else {
		return o.forbiddenError()
	}
}

/*
Ensure that `actor` is allowed to send `request` to the server.

Checks the `allow_request` rule of a policy.

If the request is permitted with an `allow_request` rule in the
policy, then this method returns `nil`. Otherwise, this method returns a
`ForbiddenError`.
*/
func (o Oso) AuthorizeRequest(actor interface{}, request interface{}) error {
	isAllowed, err := o.QueryRuleOnce("allow_request", actor, request)
	if err != nil {
		return err
	}

	if !isAllowed {
		return o.forbiddenError()
	}

	return nil
}

/*
Ensure that `actor` is allowed to perform `action` on a given
`resource`'s `field`.

Checks the `allow_field` rule of a policy.

If the action is permitted by an `allow_field` rule in the policy,
then this method returns `nil`. If the action is not permitted by the
policy, this method returns a `ForbiddenError`.
*/
func (o Oso) AuthorizeField(actor interface{}, action interface{}, resource interface{}, field interface{}) error {
	isAllowed, err := o.QueryRuleOnce("allow_field", actor, action, resource, field)
	if err != nil {
		return err
	}

	if !isAllowed {
		return o.forbiddenError()
	}

	return nil
}

/*
Return a set of actions allowed by the given (actor, resource) combination allowed
by the policy.
*/
func (o Oso) AuthorizedActions(actor interface{}, resource interface{}, allowWildcard bool) (map[interface{}]struct{}, error) {
	results := make(map[interface{}]struct{})
	query, err := (*o.p).queryRule("allow", actor, types.Variable("action"), resource)
	if err != nil {
		return nil, err
	}

	for {
		if v, err := query.Next(); err != nil {
			return nil, err
		} else if v == nil {
			break
		} else if action, ok := (*v)["action"].(interface{}); ok {
			switch val := (action).(type) {
			case types.Variable:
				if allowWildcard {
					results["*"] = struct{}{}
				} else {
					return nil, errors.New(`the result of AuthorizedActions() contained an
												"unconstrained" action that could represent any
												action, but allow_wildcard was set to False. To fix,
												set allow_wildcard to True and compare with the "*"
												string`)
				}
			default:
				results[val] = struct{}{}
			}
		}
	}
	return results, nil
}

/*
Determine the fields of `resource` on which `actor` is allowed to perform
`action`.

Uses `allow_field` rules in the policy to find all allowed fields.
*/
func (o Oso) AuthorizedFields(actor interface{}, action interface{}, resource interface{}, allowWildcard bool) (map[interface{}]struct{}, error) {
	results := make(map[interface{}]struct{})
	query, err := (*o.p).queryRule("allow_field", actor, action, resource, types.Variable("field"))
	if err != nil {
		return nil, err
	}

	for {
		if v, err := query.Next(); err != nil {
			return nil, err
		} else if v == nil {
			break
		} else if field, ok := (*v)["field"].(interface{}); ok {
			switch val := (field).(type) {
			case types.Variable:
				if allowWildcard {
					results["*"] = struct{}{}
				} else {
					return nil, errors.New(`the result of AuthorizedFields() contained an
												"unconstrained" field that could represent any
												field, but allow_wildcard was set to False. To fix,
												set allow_wildcard to True and compare with the "*"
												string`)
				}
			default:
				results[val] = struct{}{}
			}
		}
	}
	return results, nil
}

func (o Oso) SetDataFilteringAdapter(adapter types.Adapter) {
	(*o.p).host.SetDataFilteringAdapter(adapter)
}

func (o Oso) dataFilter(actor interface{}, action interface{}, resource_type string) (*Query, interface{}, error) {
	os := runtime.GOOS
	if os == "windows" {
		return nil, nil, fmt.Errorf("Data filtering is not yet supported on Windows")
	}

	query, err := (*o.p).queryRule("allow", actor, action, types.Variable("resource"))
	if err != nil {
		return nil, nil, err
	}

	query.SetAcceptExpression(true)

	constraint :=
		types.Term{
			Value: types.Value{
				types.ValueExpression{
					Operator: types.Operator{types.OperatorAnd{}},
					Args: []types.Term{
						types.Term{
							Value: types.Value{
								types.ValueExpression{
									Operator: types.Operator{types.OperatorIsa{}},
									Args: []types.Term{
										types.Term{
											Value: types.Value{
												types.ValueVariable("resource"),
											},
										},
										types.Term{
											Value: types.Value{
												types.ValuePattern{
													types.PatternInstance{
														Tag:    types.Symbol(resource_type),
														Fields: types.Dictionary{Fields: make(map[types.Symbol]types.Term)},
													},
												},
											},
										},
									},
								},
							},
						},
					},
				},
			},
		}
	query.Bind("resource", &constraint)

	partials := make([]map[string]map[string]types.Term, 0)
	for {
		if v, err := query.Next(); err != nil {
			return nil, nil, err
		} else if v == nil {
			break
		} else {
			m := make(map[string]types.Term)
			for k, v := range *v {
				polar, err := query.host.ToPolar(v)
				if err != nil {
					return nil, nil, err
				}
				m[k] = types.Term{Value: *polar}
			}
			b := make(map[string]map[string]types.Term, 0)
			b["bindings"] = m
			partials = append(partials, b)
		}
	}

	types, types_json, err := query.host.SerializeTypes()
	if err != nil {
		return nil, nil, err
	}
	filter, err := (*o.p).ffiPolar.BuildDataFilter(types_json, partials, "resource", resource_type)
	if err != nil {
		return nil, nil, err
	}
	err = query.host.ParseValues(filter)
	if err != nil {
		return nil, nil, err
	}
	filter.Types = types
	q, err := query.host.BuildQuery(filter)
	if err != nil {
		return nil, nil, err
	}
	return query, q, nil
}

/*

 */
func (o Oso) AuthorizedQuery(actor interface{}, action interface{}, resource_type string) (interface{}, error) {
	_, q, err := o.dataFilter(actor, action, resource_type)
	return q, err
}

func (o Oso) AuthorizedResources(actor interface{}, action interface{}, resource_type string) ([]interface{}, error) {
	query, q, err := o.dataFilter(actor, action, resource_type)
	if err != nil {
		return nil, err
	}
	return query.host.ExecuteQuery(q)
}

/*
Start the oso repl where you can make queries and see results printed out.
*/
func (o Oso) Repl() error {
	return (*o.p).repl()
}
