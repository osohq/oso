.. meta::
  :description: oso provides a simple REPL (Read, Evaluate, Print, Loop), so you can interactively query your oso instance.

.. _repl:

========
The REPL
========

The usual way to query an oso knowledge base is through the API in your
application's language. But especially during development and debugging,
it can be useful to interactively query a knowledge base. So oso provides
a simple REPL (Read, Evaluate, Print, Loop). To run it, first make sure
that you have :doc:`installed oso </download>`.

Once oso is installed, launch the REPL from the terminal:

.. tabs::
    .. group-tab:: Python

        .. code-block:: console
            :caption: :fab:`python` Launch the REPL

            $ python -m oso
            query>

    .. group-tab:: Ruby

        .. code-block:: console
            :caption: :fas:`gem` Launch the REPL

            $ oso
            query>

    .. group-tab:: Java

        .. code-block:: console
            :caption: :fab:`java` Launch the REPL

            $ mvn exec:java -Dexec.mainClass="com.osohq.oso.Oso"
            query>

    .. group-tab:: Node.js

        There are three ways to start the REPL depending on how you installed
        oso.

        If you installed oso globally (with ``npm install -g oso``), you should
        have an ``oso`` executable on your PATH:

        .. code-block:: console
            :caption: :fab:`node-js` Launch the REPL

            $ oso
            query>

        If you installed oso into a project and are using `Yarn
        <https://yarnpkg.com/>`_, you can run ``yarn oso`` to start the REPL:

        .. code-block:: console
            :caption: :fab:`node-js` Launch the REPL

            $ yarn oso
            query>

        .. |package_json_scripts| replace:: the ``scripts`` property of your project's package.json
        .. _package_json_scripts: https://docs.npmjs.com/files/package.json#scripts

        If you installed oso into a project and are using NPM, you can add a
        script to |package_json_scripts|_:

        .. code-block:: json
            :caption: :fab:`json` package.json

            {
              "scripts": {
                "oso": "oso"
              }
            }

        With that new script in place, ``npm run oso`` will start the REPL:

        .. code-block:: console
            :caption: :fab:`node-js` Launch the REPL

            $ npm run oso
            query>

    .. group-tab:: Rust

        To install the oso REPL, you can use ``cargo install --features=cli oso``
        to download + install it from crates.io. Or run ``cargo run --features=cli``
        from the ``languages/rust/oso`` directory in the `GitHub repository <https://github.com/osohq/oso>`_.

        .. code-block:: console
            :caption: :fab:`rust` Launch the REPL

            $ oso
            query>

.. todo:: test above

At the ``query>`` prompt, type a Polar expression and press ``Enter``.
The system responds with an answer, then prints the ``query>`` prompt
again, allowing an interactive dialog:

.. code-block:: oso

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

If the query can not be satisfied with the current knowledge base,
the response is ``false``. If the query is unconditionally true, then
the response is ``true``. Otherwise, each set of bindings that *makes*
it true is printed; e.g., the third example above has one such set,
the fourth has two.

To exit the REPL, type ``Ctrl-D`` (EOF).

Loading Policy and Application Code
===================================

To query for predicates defined in a policy, we'll need to load the
policy files. For instance, suppose we had just one ``allow`` rule for
Alice, say, in the file ``alice.polar``:

.. code-block:: polar
    :caption: :fa:`oso` alice.polar
    :class: copybutton

    allow("alice@example.com", "GET", _expense: Expense);

Then we can run the REPL, passing that filename (and any others we need)
on the command line:

.. tabs::
    .. group-tab:: Python

        .. code-block:: console
            :caption: :fab:`python` Load files and launch the REPL

            $ python -m oso alice.polar

    .. group-tab:: Ruby

        .. code-block:: console
            :caption: :fas:`gem` Load files and launch the REPL

            $ oso alice.polar

    .. group-tab:: Java

        .. code-block:: console
            :caption: :fab:`java` Load files and launch the REPL

            $ mvn exec:java -Dexec.mainClass="com.osohq.oso.Oso" -Dexec.args="alice.polar"

    .. group-tab:: Node.js

        .. code-block:: console
            :caption: :fab:`node-js` Load files and launch the REPL

            $ oso alice.polar

And now we can use the rule that was loaded:

.. TODO(gj): it's a little unfortunate that we pass in a string here instead of
   an Expense, which is the specializer in the above-loaded rule.

.. code-block:: oso

    query> allow("alice@example.com", "GET", "expense")
    true

We can also use application objects in the REPL, but we have to load
and register the defining modules before we launch the REPL. The easiest
way to do that is to write a script that imports the necessary modules,
plus ``oso``, and then use the ``Oso.repl()`` API method to start the REPL:

.. tabs::
    .. group-tab:: Python

        .. code-block:: python
            :caption: :fab:`python` app_repl.py

            from app import Expense, User

            from oso import Oso

            oso = Oso()
            oso.register_class(Expense)
            oso.register_class(User)
            oso.repl()

    .. group-tab:: Ruby

        .. code-block:: ruby
            :caption: :fas:`gem` app_repl.rb

            require 'expense'
            require 'user'

            require 'oso'

            OSO ||= Oso.new
            OSO.register_class(Expense)
            OSO.register_class(User)
            OSO.repl

    .. group-tab:: Java

        .. code-block:: java
            :caption: :fab:`java` AppRepl.java

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

    .. group-tab:: Node.js

        .. code-block:: javascript
            :caption: :fab:`node-js` app_repl.js

            const { Expense, User } = require('./models');
            const { Oso } = require('oso');

            const oso = new Oso();
            oso.registerClass(Expense);
            oso.registerClass(User);
            await oso.repl();
