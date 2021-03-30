package interfaces

/*
Interface for values that can be checked for equality
*/
type Eq interface {
	// Should return `true` when values are equal; `false` when not equal.
	Equal(other interface{}) bool
}

type Ordering int

const (
	Less    Ordering = -1
	Equal   Ordering = 0
	Greater Ordering = 1
)

func (o Ordering) Reverse() Ordering {
	switch o {
	case Less:
		return Greater
	case Equal:
		return Equal
	case Greater:
		return Less
	}
	panic("unexpected ordering")
}

/*
Interface for values that can be ordered
*/
type Ord interface {
	// Should return `true` when the comparer is less than `other`; `false` when not less than.
	Compare(other interface{}) Ordering
}

/*
Interface for values that can be iterated over.
*/
type Iterator interface {
	// Return a read-only channel of the iterator values.
	Iter() <-chan interface{}
}
