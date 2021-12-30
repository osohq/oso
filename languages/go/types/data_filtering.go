package types

import (
	"encoding/json"
)

type Comparison int

const (
	Eq Comparison = iota
	Neq
	In
)

func (comparison *Comparison) UnmarshalJSON(b []byte) error {
	var cmp string
	err := json.Unmarshal(b, &cmp)
	if err != nil {
		return err
	}
	switch cmp {
	case "Eq":
		*comparison = Eq
	case "Neq":
		*comparison = Neq
	case "In":
		*comparison = In
	}
	return nil
}

type Projection struct {
	TypeName  string
	FieldName string
}

func (proj *Projection) UnmarshalJSON(b []byte) error {
	var l []string
	err := json.Unmarshal(b, &l)
	if err != nil {
		return err
	}
	proj.TypeName = l[0]
	proj.FieldName = l[1]
	return nil
}

type Immediate struct {
	Value interface{}
}

type DatumVariant interface {
	isDatum()
}

type Datum struct {
	DatumVariant
}

func (datum *Datum) UnmarshalJSON(b []byte) error {
	var m map[string]*json.RawMessage
	err := json.Unmarshal(b, &m)
	if err != nil {
		return err
	}
	for k, v := range m {
		switch k {
		case "Immediate":
			var val Value
			err = json.Unmarshal(*v, &val)
			if err != nil {
				return err
			}
			datum.DatumVariant = Immediate{val}
		case "Field":
			var proj Projection
			err = json.Unmarshal(*v, &proj)
			if err != nil {
				return err
			}
			datum.DatumVariant = proj
		}
		break
	}
	return nil
}

func (Projection) isDatum() {}
func (Immediate) isDatum()  {}

type FilterRelation struct {
	FromTypeName  string
	FromFieldName string
	ToTypeName    string
}

func (relation *FilterRelation) UnmarshalJSON(b []byte) error {
	var fields []string
	err := json.Unmarshal(b, &fields)
	if err != nil {
		return err
	}
	relation.FromTypeName = fields[0]
	relation.FromFieldName = fields[1]
	relation.ToTypeName = fields[2]
	return nil
}

type FilterCondition struct {
	Lhs Datum
	Cmp Comparison
	Rhs Datum
}

func (relation *FilterCondition) UnmarshalJSON(b []byte) error {
	var fields []*json.RawMessage

	err := json.Unmarshal(b, &fields)
	if err != nil {
		return err
	}
	var lhs Datum
	err = json.Unmarshal(*fields[0], &lhs)
	if err != nil {
		return err
	}
	var op Comparison
	err = json.Unmarshal(*fields[1], &op)
	if err != nil {
		return err
	}
	var rhs Datum
	err = json.Unmarshal(*fields[2], &rhs)
	if err != nil {
		return err
	}
	relation.Lhs = lhs
	relation.Cmp = op
	relation.Rhs = rhs
	return nil
}

type Filter struct {
	// Root
	Root string `json:"root"`
	// Relations
	Relations []FilterRelation `json:"relations"`
	// Conditions
	Conditions [][]FilterCondition `json:"conditions"`
	// Types
	Types map[string]map[string]interface{}
}
