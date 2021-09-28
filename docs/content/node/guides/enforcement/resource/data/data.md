---
authorize: authorize
isAdmin: isAdmin
forbiddenError: ForbiddenError
notFoundError: NotFoundError
authorizeLink: |-
  <a href="https://docs-preview.oso.dev/node/reference/api/classes/oso.oso-1.html#authorize">`Oso.authorize`</a>
forbiddenErrorLink: |-
  <a href="/node/reference/api/classes/errors.forbiddenerror.html">`ForbiddenError`</a>
notFoundErrorLink: |-
  <a href="/node/reference/api/classes/errors.notfounderror.html">`NotFoundError`</a>
authorizationErrorLink: |-
  <a href="/node/reference/api/classes/errors.authorizationerror.html">an `AuthorizationError`</a>
exampleCall: |-
    ```javascript
    oso.authorize(user, "approve", expense)
    ```
approveExpense: |-
    ```javascript
    async function approveExpense(user, expenseId) {
        const expense = await db.fetch(
            "SELECT * FROM expenses WHERE id = %", expenseId);
        await oso.authorize(user, "approve", expense);

        // ... process request
    }
    ```
globalErrorHandler: |-
    As an example, here's what a global error handler looks like in an Express app:

    ```javascript
    import { ForbiddenError, NotFoundError } from "oso";

    function handleError(err, req, res, next) {
      if (res.headersSent) {
        return next(err)
      }
      if (err instanceof ForbiddenError) {
        res.status(403).send("Forbidden");
      } else if (err instanceof NotFoundError) {
        res.status(404).send("Not found");
      } else {
        // Handle other errors
      }
    }
    app.use(handleError);
    ```

    Then, when each route uses the `authorize` method, the app knows how to
    respond when an authorization error occurs.
---
