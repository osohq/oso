package types

type Relation struct {
	Kind       string // maybe this should be an enum
	OtherType  string
	MyField    string
	OtherField string
}
