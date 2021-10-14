---
startRepl: |
    ```
    $ python -m oso
    query>
    ```
startReplWithFile: |
    ```
    $ python -m oso alice.polar
    ```
replApi: |
    ```python
    from app import Expense, User

    from oso import Oso

    oso = Oso()
    oso.register_class(Expense)
    oso.register_class(User)
    oso.repl()
    ```
---