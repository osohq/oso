---
submitted_by: submittedBy
postfixId: Id

expenseClass: |
    ```js
    class Expense {
      constructor({ amount, submitted_by, location, project_id }) {
        // ...
    }

    oso.registerClass(Expense);
    ```

userClass: |
    ```js
    class User {
      constructor(name, location) {
        // ...
    }

    oso.registerClass(User);
    ```
---
