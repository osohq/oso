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
