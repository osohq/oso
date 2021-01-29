package oso

type Oso struct {
	p *Polar
}

func NewOso() (Oso, error) {
	if p, e := NewPolar(); e != nil {
		return Oso{}, e
	} else {
		return Oso{p: p}, nil
	}
}

func (o Oso) LoadFile(f string) error {
	return (*o.p).LoadFile(f)
}

func (o Oso) LoadString(s string) error {
	return (*o.p).LoadString(s)
}

func (o Oso) ClearRules() error {
	return (*o.p).ClearRules()
}

/*
Query the policy using a query string; the query is run in a new Go routine.
Accepts the string to query for.
Returns a channel of resulting binding maps, and a channel for errors.
As the query is evaluated, all resulting bindings will be written to the results channel,
and any errors will be written to the error channel.
*/
func (o Oso) QueryStr(q string) (<-chan map[string]interface{}, <-chan error) {
	if query, err := (*o.p).QueryStr(q); err != nil {
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
	if query, err := (*o.p).QueryRule(name, args...); err != nil {
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
	return (*o.p).QueryStr(q)
}

/*
Create policy query for a rule.
Accepts the name of the rule to query, and a variadic list of rule arguments.
Returns a new *Query, on which `Next()` can be called to get the next result,
or an error.
*/
func (o Oso) NewQueryFromRule(name string, args ...interface{}) (*Query, error) {
	return (*o.p).QueryRule(name, args...)
}

/*
Check if an (actor, action, resource) combination is allowed by the policy.
Returns the result as a bool, or an error.
*/
func (o Oso) IsAllowed(actor interface{}, action interface{}, resource interface{}) (bool, error) {
	query, err := (*o.p).QueryRule("allow", actor, action, resource)
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
