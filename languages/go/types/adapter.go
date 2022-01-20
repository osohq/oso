package types

type Adapter interface {
	BuildQuery(*Filter) (interface{}, error)
	ExecuteQuery(interface{}) ([]interface{}, error)
}
