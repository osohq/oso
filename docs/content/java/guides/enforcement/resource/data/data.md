---
authorize: authorize
isAdmin: isAdmin
forbiddenError: ForbiddenException
notFoundError: NotFoundException
# TODO: change these to relative links
authorizeLink: |-
  [`Oso.authorize`](https://docs.osohq.com/java/reference/api/com/osohq/oso/Oso.html#authorize(java.lang.Object,java.lang.Object,java.lang.Object,boolean))
forbiddenErrorLink: |-
  [`ForbiddenException`](https://docs.osohq.com/java/reference/api/com/osohq/oso/Exceptions.ForbiddenException.html)
notFoundErrorLink: |-
  [`NotFoundException`](https://docs.osohq.com/java/reference/api/com/osohq/oso/Exceptions.NotFoundException.html)
authorizationErrorLink: |-
  an [`AuthorizationException`](https://docs.osohq.com/java/reference/api/com/osohq/oso/Exceptions.AuthorizationException.html)
exampleCall: |-
    ```java
    oso.authorize(user, "approve", expense);
    ```
approveExpense: |-
    ```java
    public void approveExpense(User user, int expenseId) throws AuthorizationException {
        Expense expense = Expense.byId(expenseId);
        oso.authorize(user, "approve", expense);

        // ... process request
    }
    ```
globalErrorHandler: |-
    As an example, here's what a global exception handler looks like in a Spring MVC app:

    ```java
    import com.osohq.oso.Exceptions;

    @ControllerAdvice
    class GlobalControllerAuthorizationExceptionHandler {
        @ResponseStatus(HttpStatus.NOT_FOUND) // 404
        @ExceptionHandler(Exceptions.NotFoundException.class)
        public void handleOsoNotFound() {}

        @ResponseStatus(HttpStatus.FORBIDDEN) // 403
        @ExceptionHandler(Exceptions.ForbiddenException.class)
        public void handleOsoForbidden() {}
    }
    ```

    Then, when your application calls `authorize`, it
    will know how to handle exceptions that arise.
---
