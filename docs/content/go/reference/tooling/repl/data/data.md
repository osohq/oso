---
startRepl: |
    To use the Go REPL, first build the executable from within the `oso-go` library:

    ```
    $ go build cmd/oso/oso.go
    ```

    Then, you can run the REPL:

    ```
    $ ./oso
    query>
    ```
startReplWithFile: |
    ```
    $ ./oso alice.polar
    ```
replApi: |
    ```go
    import "github.com/osohq/go-oso"

    func main() {
        // error handling not shown
        osoInstance, err := oso.NewOso()
        osoInstance.RegisterClass(reflect.TypeOf(Expense{}), nil)
        osoInstance.RegisterClass(reflect.TypeOf(User{}), nil)
        osoInstance.LoadFile("policy.polar")
        osoInstance.Repl()
    }
    ```
---