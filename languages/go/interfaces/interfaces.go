package interfaces

type Comparer interface {
	Equal(other Comparer) bool
	Lt(other Comparer) bool
}

type Iterator interface {
	Iter() chan interface{}
}
