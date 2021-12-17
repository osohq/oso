package types

type Adapter interface {
	BuildQuery(*Filter) (interface{}, error)
	ExecQuery(interface{}) (interface{}, error)
}
