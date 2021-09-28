package main

import (
	"fmt"
	"os"

	"github.com/osohq/go-oso"
)

func main() {
	oso_instance, err := oso.NewOso()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	filenames := os.Args[1:]
	err = oso_instance.LoadFiles(filenames)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	err = oso_instance.Repl()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
