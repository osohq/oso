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

        .. code-block:: console
            :caption: :fab:`node-js` Launch the REPL

            $ ./node_modules/.bin/oso
            query>

        .. tip::

           Install oso globally with ``npm install -g`` if you'd like to have
           the ``oso`` command on your path instead of in your project's
           ``node_modules`` directory.

           If you are using ``yarn``, ``yarn oso`` will launch the repl.

.. todo:: test above

At the ``query>`` prompt, type a Polar expression and press ``Enter``.
The system responds with an answer, then prints the ``query>`` prompt
again, allowing an interactive dialog:

.. code-block:: oso

    query> 1 = 1
    True
    query> 1 = 2
    False
    query> x = 1 and y = 2
    {x=1, y=2}
    query> x = 1 or x = 2
    {x=1}
    {x=2}
    query> x = 1 and x = 2
    false
    query>

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

.. literalinclude:: /examples/quickstart/polar/expenses-02.polar
    :caption: :fa:`oso` alice.polar
    :class: copybutton

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

            $ ./node_modules/.bin/oso -- alice.polar

And now we can use the rule that was loaded:

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

            const { Expense, User } = require("./models");
            const { Oso } = require("oso");

            const oso = new Oso();
            oso.registerClass(Expense);
            oso.registerClass(User);
            await oso.repl();
