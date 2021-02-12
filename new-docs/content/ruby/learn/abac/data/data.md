---
submitted_by: submitted_by
postfixId: \_id

expenseClass: |
    ```ruby
    class Expense
      ...
    end

    OSO.register_class(Expense)
    ```

userClass: |
    ```ruby
    OSO ||= Oso.new

    class User
      ...
    end

    OSO.register_class(User)
    ```
---
