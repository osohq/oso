type User struct {
	...
}

...

oso.RegisterClass(reflect.TypeOf(User{}))