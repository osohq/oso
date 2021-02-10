package main

import (
	"fmt"

	"github.com/osohq/go-oso"
)

func main() {
	oso_instance, err := oso.NewOso()
	if err != nil {
		fmt.Println(err)
	}
	err = oso_instance.Repl()
	if err != nil {
		fmt.Println(err)
	}
}
