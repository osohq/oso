---
authorize: Authorize
isAdmin: IsAdmin
forbiddenError: ForbiddenError
notFoundError: NotFoundError
# TODO: change these to relative links
authorizeLink: |-
  [`Oso.authorize`](https://pkg.go.dev/github.com/osohq/go-oso#Oso.Authorize)
forbiddenErrorLink: |-
  [`ForbiddenError`](https://pkg.go.dev/github.com/osohq/go-oso/errors#ForbiddenError)
notFoundErrorLink: |-
  [`NotFoundError`](https://pkg.go.dev/github.com/osohq/go-oso/errors#NotFoundError)
authorizationErrorLink: |-
  an [`AuthorizationError`](https://pkg.go.dev/github.com/osohq/go-oso/errors)
exampleCall: |-
    ```go
    oso.Authorize(user, "approve", expense)
    ```
getExpense: |-
    ```go
    func GetExpense(user User, expenseId int) {
        expense := db.Fetch(
            "SELECT * FROM expenses WHERE id %", expenseId)
        oso.Authorize(user, "read", "expense")

        // ... process request
    }
    ```
globalErrorHandler: ""
---
