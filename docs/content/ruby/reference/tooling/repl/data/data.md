---
startRepl: |
    ```
    $ oso
    query>
    ```
startReplWithFile: |
    ```
    $ oso alice.polar
    ```
replApi: |
    ```ruby
    require 'expense'
    require 'user'

    require 'oso'

    OSO ||= Oso.new
    OSO.register_class(Expense)
    OSO.register_class(User)
    OSO.load_files(["alice.polar"])
    OSO.repl
    ```
---