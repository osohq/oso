---
githubApp: "[go sample app](https://github.com/osohq/oso-go-quickstart)"
githubURL: "https://github.com/osohq/oso-go-quickstart.git"
installation: |
    Install the project dependencies, then run the server:
    ```bash
    $ go get github.com/osohq/go-oso
    installing requirements
    
    $ go run quickstart.go
    server running on port 5050
    ```
submitted_by: SubmittedBy
endswith: EndsWith
amount: Amount
manager: Manager
endswithURL: >
   [the `EndsWith` method](https://github.com/osohq/oso-go-quickstart/blob/main/quickstart.go#L16-L18)
expenses1: |
  allow(actor: String, "GET", _expense: Expense) if
      actor.EndsWith("@example.com");
expenses2: |
  allow(actor: String, "GET", expense: Expense) if
      expense.SubmittedBy = actor;
isAllowed: IsAllowed
---
