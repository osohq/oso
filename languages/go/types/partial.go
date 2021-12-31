package types

import "encoding/json"

type Expression struct {
	Operator Operator
	Args     []interface{}
}

type Variable string

type TypeFields map[string]Type
type TypeMap map[string]TypeFields

func (v TypeFields) MarshalJSON() ([]byte, error) {
	intermediate := make(map[string]TypeDeserializer)
	for k, v := range v {
		intermediate[k] = TypeDeserializer{Type: v}
	}
	return json.Marshal(intermediate)
}

func (res *TypeFields) UnmarshalJSON(b []byte) error {
	var intermediate map[string]TypeDeserializer
	err := json.Unmarshal(b, &intermediate)
	if err != nil {
		return err
	}
	*res = make(TypeFields)
	for k, v := range intermediate {
		(*res)[k] = v.Type
	}
	return nil
}
