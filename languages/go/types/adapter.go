package types

type Adapter interface {
	BuildQuery(TypeMap, *Filter) (interface{}, error)
	ExecuteQuery(interface{}) (interface{}, error)
}
