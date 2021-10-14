package interfaces

/*
Interface for values that can be compared.
*/
type Comparer interface {
	// Should return `true` when values are equal; `false` when not equal.
	Equal(other interface{}) bool
	// Should return `true` when the comparer is less than `other`; `false` when not less than.
	Lt(other interface{}) bool
}

/*
Interface for values that can be iterated over.
*/
type Iterator interface {
	// Return a read-only channel of the iterator values.
	Iter() <-chan interface{}
}
