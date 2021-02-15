---
title: REPL
aliases:
    - ../more/dev-tools/repl.html
description: Oso provides a simple REPL (Read, Evaluate, Print Loop) to interactively query a policy.
---

# REPL

The usual way to query an Oso knowledge base is through the API in your
application’s language. But especially during development and debugging,
it can be useful to interactively query a policy. So Oso provides
a simple REPL (Read, Evaluate, Print, Loop). To run it, first make sure
that you have installed Oso.

Once Oso is installed, launch the REPL from the terminal:

Python

```
$ python -m oso
query>
```

Ruby

```
$ oso
query>
```

Java

```
$ mvn exec:java -Dexec.mainClass="com.osohq.oso.Oso"
query>
```

Node.js

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
script to [the `scripts` property of your project’s package.json](https://docs.npmjs.com/files/package.json#scripts):

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

Rust

To install the Oso REPL, you can use `cargo install --features=cli oso`
to download + install it from crates.io. Or run `cargo run --features=cli`
from the `languages/rust/oso` directory in the [GitHub repository](https://github.com/osohq/oso).

```
$ oso
query>
```

At the `query>` prompt, type a Polar expression and press `Enter`.
The system responds with an answer, then prints the `query>` prompt
again, allowing an interactive dialog:

```
query> 1 = 1
true
query> 1 = 2
false
query> x = 1 and y = 2
y => 2
x => 1
query> x = 1 or x = 2
x => 1
x => 2
query> x = 1 and x = 2
false
```

If the query can not be satisfied with the current knowledge base,
the response is `false`. If the query is unconditionally true, then
the response is `true`. Otherwise, each set of bindings that *makes*
it true is printed; e.g., the third example above has one such set,
the fourth has two.

To exit the REPL, type `Ctrl-D` (EOF).

## Loading Policy and Application Code

To query for predicates defined in a policy, we’ll need to load the
policy files. For instance, suppose we had just one `allow` rule for
Alice, say, in the file `alice.polar`:

```
allow("alice@example.com", "GET", _expense: Expense);
```

Then we can run the REPL, passing that filename (and any others we need)
on the command line:

Python

```
$ python -m oso alice.polar
```

Ruby

```
$ oso alice.polar
```

Java

```
$ mvn exec:java -Dexec.mainClass="com.osohq.oso.Oso" -Dexec.args="alice.polar"
```

Node.js

```
$ oso alice.polar
```

And now we can use the rule that was loaded:

<!-- TODO(gj): it's a little unfortunate that we pass in a string here instead of
an Expense, which is the specializer in the above-loaded rule. -->
```
query> allow("alice@example.com", "GET", "expense")
true
```

We can also use application objects in the REPL, but we have to load
and register the defining modules before we launch the REPL. The easiest
way to do that is to write a script that imports the necessary modules,
plus `oso`, and then use the `Oso.repl()` API method to start the REPL:

Python

```
from app import Expense, User

from oso import Oso

oso = Oso()
oso.register_class(Expense)
oso.register_class(User)
oso.repl()
```

Ruby

```
require 'expense'
require 'user'

require 'oso'

OSO ||= Oso.new
OSO.register_class(Expense)
OSO.register_class(User)
OSO.repl
```

Java

```
import com.example.Expense;
import com.example.User;

import com.osohq.oso.*;

public class AppRepl {
    public static void main(String[] args) throws OsoException, IOException {
        Oso oso = new Oso();
        oso.registerClass(Expense.class);
        oso.registerClass(User.class);
        oso.repl(args)
    }
}
```

Node.js

```
const { Expense, User } = require('./models');
const { Oso } = require('oso');

const oso = new Oso();
oso.registerClass(Expense);
oso.registerClass(User);
await oso.repl();
```
