---
authorize: authorize
isAdmin: is_admin
forbiddenError: ForbiddenError
notFoundError: NotFoundError
authorizeLink: |-
  `Oso.authorize`
forbiddenErrorLink: |-
  `Oso::ForbiddenError`
notFoundErrorLink: |-
  `Oso::NotFoundError`
authorizationErrorLink: |-
  an `Oso::AuthorizationError`
getExpense: |-
    ```ruby
    def get_expense(user, expense_id)
      expense = db.fetch(
        "SELECT * FROM expenses WHERE id = ?", expense_id)
      oso.authorize(user, "read", expense)

      # ... process request
    end
    ```
globalErrorHandler: |-
    As an example, here's what a global error handler looks like in a Rails app:

    ```ruby
    class ApplicationController < ActionController::Base
      rescue_from Oso::ForbiddenError, with: :forbidden_error
      rescue_from Oso::NotFoundError, with: :not_found

      def forbidden_error
        render 'forbidden', status: 403
      end

      def not_found_error
        render 'not found', status: 404
      end
    end
    ```

    Then, when your controllers call `authorize`, they don't need to worry about
    handling the errors.
---
