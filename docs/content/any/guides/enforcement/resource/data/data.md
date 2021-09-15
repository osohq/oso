---
authorize: authorize
isAdmin: is_admin
forbiddenError: ForbiddenError
notFoundError: NotFoundError
# TODO: change these to relative links
authorizeLink: |-
  [`Oso.authorize`](https://docs.osohq.com/python/reference/api/index.html#oso.Oso.authorize)
forbiddenErrorLink: |-
  [`ForbiddenError`](https://docs.osohq.com/python/reference/api/index.html#oso.exceptions.ForbiddenError)
notFoundErrorLink: |-
  [`NotFoundError`](https://docs.osohq.com/python/reference/api/index.html#oso.exceptions.NotFoundError)
authorizationErrorLink: |-
  an [`AuthorizationError`](https://docs.osohq.com/python/reference/api/index.html#oso.exceptions.AuthorizationError)
exampleCall: |-
    ```python
    oso.authorize(user, "approve", expense)
    ```
approveExpense: |-
    ```python
    def approve_expense(user, expense_id):
        expense = db.fetch(
            "SELECT * FROM expenses WHERE id = %", expense_id)
        oso.authorize(user, "approve", expense)

        # ... process request
    ```
globalErrorHandler: |-
    As an example, here's what a global error handler looks like in a Flask app:

    ```python
    from oso import ForbiddenError, NotFoundError

    app = Flask()

    @app.errorhandler(ForbiddenError)
    def handle_forbidden(*_):
        return {"message": "Forbidden"}, 403

    @app.errorhandler(NotFoundError)
    def handle_not_found(*_):
        return {"message": "Not Found"}, 404
    ```

    Then, when your application calls `authorize`, it
    will know how to handle errors that arise.
---
