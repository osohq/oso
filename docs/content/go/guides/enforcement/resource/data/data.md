---
authorize: Authorize
isAdmin: IsAdmin
forbiddenError: ForbiddenError
notFoundError: NotFoundError
# TODO: change these to relative links
authorizeLink: |-
  [`Oso.Authorize`](https://pkg.go.dev/github.com/osohq/go-oso#Oso.Authorize)
forbiddenErrorLink: |-
  [`ForbiddenError`](https://pkg.go.dev/github.com/osohq/go-oso/errors#ForbiddenError)
notFoundErrorLink: |-
  [`NotFoundError`](https://pkg.go.dev/github.com/osohq/go-oso/errors#NotFoundError)
authorizationErrorLink: |-
  an authorization error
exampleCall: |-
    ```go
    err := oso.Authorize(user, "approve", expense)
    ```
approveExpense: |-
    ```go
    func ApproveExpense(user User, expenseId int) {
        expense := db.Fetch(
            "SELECT * FROM expenses WHERE id %", expenseId)
        if err := oso.Authorize(user, "approve", expense); err != nil {
            // handle error
        }

        // ... process request
    }
    ```
globalErrorHandler: ""
---
