package oso

/*
The central object to manage policy state and verify requests.
*/
type Oso struct {
	p *Polar
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
		return Oso{p: p}, nil
	}
}

/*
Load Polar policy from a ".polar" file, checking that all inline queries succeed.
*/
func (o Oso) LoadFile(f string) error {
	return (*o.p).loadFile(f)
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
	return (*o.p).registerClass(cls, ctor, nil)
}

/*
Register a Go type under a certain name/alias so that it can be referenced
within Polar files by that name. Accepts a concrete value of the Go type and a
constructor function or nil if no constructor is required.
*/
func (o Oso) RegisterClassWithName(cls interface{}, ctor interface{}, name string) error {
	return (*o.p).registerClass(cls, ctor, &name)
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
	query, err := (*o.p).queryRule("allow", actor, action, resource)
	if err != nil {
		return false, err
	}
	results, err := query.Next()
	if err != nil {
		return false, err
	} else if results != nil {
		return true, nil
	} else {
		return false, nil
	}
}

/*
Start the oso repl where you can make queries and see results printed out.
*/
func (o Oso) Repl() error {
	return (*o.p).repl()
}
