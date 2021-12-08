---
startRepl: |
    There are three ways to start the REPL depending on how you installed
    Oso.

    If you installed Oso globally (with `npm install -g oso`), you should
    have an `oso` executable on your PATH:

    ```
    $ oso
    query>
    ```

    If you installed Oso into a project and are using [Yarn](https://yarnpkg.com/), you can run `yarn oso` to start the REPL:

    ```
    $ yarn oso
    query>
    ```

    If you installed Oso into a project and are using NPM, you can add a
    script to [the `scripts` property of your project’s `package.json`](https://docs.npmjs.com/files/package.json#scripts):

    ```
    {
    "scripts": {
        "oso": "oso"
    }
    }
    ```

    With that new script in place, `npm run oso` will start the REPL:

    ```
    $ npm run oso
    query>
    ```
startReplWithFile: |
    ```
    $ oso alice.polar
    ```
replApi: |
    ```javascript
    const { Expense, User } = require('./models');
    const { Oso } = require('oso');

    const oso = new Oso();
    oso.registerClass(Expense);
    oso.registerClass(User);
    oso.loadFiles(["alice.polar"])
    await oso.repl();
    ```
---
