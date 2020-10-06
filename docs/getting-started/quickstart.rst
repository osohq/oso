==========
Quickstart
==========

oso helps developers build authorization into their applications. If you've
never used oso before and want to see it in action, this guide is for you.
We're going to walk through how to use oso to add authorization to a simple web
server.

.. admonition:: Try it!
    :class: tip try-it

    .. tabs::
        .. group-tab:: Python

            To follow along, clone `the sample app
            <https://github.com/osohq/oso-python-quickstart>`_:

            .. code-block:: console
                :class: copybutton

                $ git clone https://github.com/osohq/oso-python-quickstart.git

        .. group-tab:: Ruby

                To follow along, clone `the sample app
                <https://github.com/osohq/oso-ruby-quickstart>`_:

                .. code-block:: console
                    :class: copybutton

                    $ git clone https://github.com/osohq/oso-ruby-quickstart.git

        .. group-tab:: Java

            To follow along, clone `the sample app
            <https://github.com/osohq/oso-java-quickstart>`_:

            .. code-block:: console
                :class: copybutton

                $ git clone https://github.com/osohq/oso-java-quickstart.git

        .. group-tab:: Node.js

            To follow along, clone `the sample app
            <https://github.com/osohq/oso-nodejs-quickstart>`_:

            .. code-block:: console
                :class: copybutton

                $ git clone https://github.com/osohq/oso-nodejs-quickstart.git

        .. group-tab:: Rust

            To follow along, clone `the sample app
            <https://github.com/osohq/oso-rust-quickstart>`_:

            .. code-block:: console
                :class: copybutton

                $ git clone https://github.com/osohq/oso-rust-quickstart.git

Run the server
==============

Our sample application serves data about expenses submitted by users. The
sample application has three important files.

One file defines a simple ``Expense`` class and some sample data stored in a
map.

A second file has our HTTP server code, where we have defined a route handler
for ``GET`` requests to the path ``/expenses/:id``. We've already added an
authorization check using the :doc:`oso library </using/libraries/index>` to
control access to expense resources. You can learn more about how to add oso to
your application :doc:`here </getting-started/application/index>`.

The third file is the oso policy file, ``expenses.polar``, and is currently
empty.

.. admonition:: Try it!
    :class: tip try-it

    .. tabs::
        .. group-tab:: Python

            Install the project dependencies with pip, then run the server:

            .. code-block:: console
                :class: copybutton

                $ pip install -r requirements.txt

            .. code-block:: console
                :class: copybutton

                $ python quickstart/server.py
                server running on port 5050

        .. group-tab:: Ruby

            Install the project dependencies with Bundler, then run the server:

            .. code-block:: console
                :class: copybutton

                $ bundle install

            .. code-block:: console
                :class: copybutton

                $ bundle exec ruby server.rb
                [2020-10-06 12:22:26] INFO  WEBrick 1.6.0
                [2020-10-06 12:22:26] INFO  ruby 2.7.0 (2019-12-25) [x86_64-darwin19]
                [2020-10-06 12:22:26] INFO  WEBrick::HTTPServer#start: pid=47366 port=5050

        .. group-tab:: Java

            Go to the `Maven Repository
            <https://search.maven.org/artifact/com.osohq/oso>`_ to install oso.
            Either add oso as a dependency to your build system, or download
            the latest JAR file and add it to your Java project libraries.

            Once you have oso installed, run ``Server.java``.

        .. group-tab:: Node.js

            Install the project dependencies with NPM (or Yarn), then run the
            server:

            .. code-block:: console
                :class: copybutton

                $ npm install

            .. code-block:: console
                :class: copybutton

                $ npm start
                server running on port 5050

        .. group-tab:: Rust

            Run the server with Cargo, which will also install project
            dependencies:

            .. code-block:: console
                :class: copybutton

                $ cargo run

        With the server running, open a
        second terminal and make a request using cURL:

        .. code-block:: console
            :class: copybutton

            $ curl localhost:8000/expenses/1
            Not Authorized!

You'll get a "Not Authorized!" response because we haven't added any rules to
our oso policy (in ``expenses.polar``), and oso is deny-by-default.

Let's start implementing our access control scheme by adding some rules to the
oso policy.

Adding our first rule
=====================

oso rules are written in a declarative policy language called Polar. You can
include any kind of rule in a policy, but the oso library is designed to
evaluate :ref:`allow rules <allow-rules>`, which specify the conditions that
allow an **actor** to perform an **action** on a **resource**.

.. admonition:: Edit it!
    :class: note

    In our policy file (``expenses.polar``), let's add a rule that allows
    anyone with an email ending in ``"@example.com"`` to view all expenses:

    .. tabs::
        .. group-tab:: Python

            .. literalinclude:: /examples/quickstart/polar/expenses-01-python.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

            .. |str_endswith| replace:: the ``str.endswith`` method
            .. _str_endswith: https://docs.python.org/3/library/stdtypes.html#str.endswith

            Note that the call to ``.endswith`` is actually calling out to
            |str_endswith|_ defined in the Python standard library. The
            **actor** value passed to oso is a Python string, and oso allows
            us to call methods from Python's standard library on it.

        .. group-tab:: Ruby

            .. literalinclude:: /examples/quickstart/polar/expenses-01-ruby.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

            .. |string_end_with| replace:: the ``String#end_with?`` method
            .. _string_end_with: https://ruby-doc.org/core/String.html#method-i-end_with-3F

            Note that the call to ``.end_with?`` is actually calling out to
            |string_end_with|_ defined in the Ruby standard library. The
            **actor** value passed to oso is a Ruby string, and oso allows
            us to call methods from Ruby's standard library on it.

        .. group-tab:: Java

            .. literalinclude:: /examples/quickstart/polar/expenses-01-java.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

            .. |string_endsWithJava| replace:: the ``String.endsWith`` method
            .. _string_endsWithJava: https://docs.oracle.com/javase/10/docs/api/java/lang/String.html#endsWith(java.lang.String)

            Note that the call to ``.endsWith`` is actually calling out to
            |string_endsWithJava|_ defined in the Java standard library. The
            **actor** value passed to oso is a Java string, and oso allows
            us to call methods from Java's standard library on it.

        .. group-tab:: Node.js

            .. literalinclude:: /examples/quickstart/polar/expenses-01-nodejs.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

            .. |string_endsWithJS| replace:: the ``String.prototype.endsWith`` method
            .. _string_endsWithJS: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/endsWith

            Note that the call to ``.endsWith`` is actually calling out to
            |string_endsWithJS|_ defined in the JavaScript standard library.
            The **actor** value passed to oso is a JavaScript string, and oso
            allows us to call methods from JavaScript's standard library on
            it.

        .. group-tab:: Rust

            .. literalinclude:: /examples/quickstart/polar/expenses-01-rust.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

            .. |string_endsWithRust| replace:: the ``String::ends_with`` method
            .. _string_endsWithRust: https://doc.rust-lang.org/std/string/struct.String.html#method.ends_with

            Note that the call to ``.ends_with`` is actually calling out to
            |string_endsWithRust|_ defined in the Rust standard library. The
            **actor** value passed to oso is a Rust string, and oso allows us
            to call methods from Rust's standard library on it.

The ``Expense`` and ``String`` terms following the colons in the head of the
rule are :ref:`specializers <Specialization>`, patterns that control rule
execution based on whether they match the supplied argument. This syntax
ensures that the rule will only be evaluated when the actor is a string and
the resource is an instance of the ``Expense`` class.

.. admonition:: Try it!
    :class: tip try-it

    Once we've added our new rule and restarted the web server, every user with
    an ``@example.com`` email should be allowed to view any expense:

    .. code-block:: console
        :class: copybutton

        $ curl -H "user: alice@example.com" localhost:8000/expenses/1
        Expense(...)

Okay, so what just happened?

When we ask oso for a policy decision via ``Oso.is_allowed()``, the oso engine
searches through its knowledge base to determine whether the provided
**actor**, **action**, and **resource** satisfy any **allow** rules. In the
above case, we passed in ``"alice@example.com"`` as the **actor**, ``"GET"`` as
the **action**, and the ``Expense`` object with ``id=1`` as the **resource**.
Since ``"alice@example.com"`` ends with ``@example.com``, our rule is
satisfied, and Alice is allowed to view the requested expense.

.. admonition:: Try it!
    :class: tip try-it

    If a user's email doesn't end in ``"@example.com"``, the rule fails, and
    they are denied access:

    .. code-block:: console
        :class: copybutton

        $ curl -H "user: alice@foo.com" localhost:8000/expenses/1
        Not Authorized!

If you aren't seeing the same thing, make sure you created your policy
correctly in ``expenses.polar``.

Using application data
======================

We now have some basic access control in place, but we can do better.
Currently, anyone with an email ending in ``@example.com`` can see all expenses
-- including expenses submitted by others.

.. admonition:: Edit it!
    :class: note

    Let's modify our existing rule such that users can only see their own
    expenses:

    .. tabs::
        .. group-tab:: Python

            .. literalinclude:: /examples/quickstart/polar/expenses-02-python.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

        .. group-tab:: Ruby

            .. literalinclude:: /examples/quickstart/polar/expenses-02-ruby.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

        .. group-tab:: Java

            .. literalinclude:: /examples/quickstart/polar/expenses-02-java.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

        .. group-tab:: Node.js

            .. literalinclude:: /examples/quickstart/polar/expenses-02-nodejs.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

        .. group-tab:: Rust

            .. literalinclude:: /examples/quickstart/polar/expenses-02-rust.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

Behind the scenes, oso looks up the ``submitted_by`` field on the provided
``Expense`` instance and compares that value against the provided **actor**.
And just like that, an actor can only see an expense if they submitted it!

.. admonition:: Try it!
    :class: tip try-it

    Alice can see her own expenses but not Bhavik's:

    .. code-block:: console
        :class: copybutton

        $ curl -H "user: alice@example.com" localhost:8000/expenses/1
        Expense(...)

    .. code-block:: console
        :class: copybutton

        $ curl -H "user: alice@example.com" localhost:8000/expenses/3
        Not Authorized!


Feel free to play around with the current policy and experiment with adding
your own rules!

For example, if you have ``Expense`` and ``User`` classes defined in your
application, you could write a policy rule in oso that says a ``User`` may
``"approve"`` an ``Expense`` if they manage the ``User`` who submitted the
expense and the expense's amount is less than $100.00:

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :class: no-select

            allow(approver: User, "approve", expense: Expense) if
                approver = expense.submitted_by.manager
                and expense.amount < 10000;

    .. group-tab:: Ruby

        .. code-block:: polar
            :class: no-select

            allow(approver: User, "approve", expense: Expense) if
                approver = expense.submitted_by.manager
                and expense.amount < 10000;

    .. group-tab:: Java

        .. code-block:: polar
            :class: no-select

            allow(approver: User, "approve", expense: Expense) if
                approver = expense.submittedBy.manager
                and expense.amount < 10000;

    .. group-tab:: Node.js

        .. code-block:: polar
            :class: no-select

            allow(approver: User, "approve", expense: Expense) if
                approver = expense.submittedBy.manager
                and expense.amount < 10000;

    .. group-tab:: Rust

        .. code-block:: polar
            :class: no-select

            allow(approver: User, "approve", expense: Expense) if
                approver = expense.submitted_by.manager
                and expense.amount < 10000;

In the process of evaluating that rule, the oso engine would call back into the
application in order to make determinations that rely on application data, such
as:

- Which user submitted the expense in question?
- Who is their manager?
- Is their manager the user who's attempting to approve the expense?
- Does the expense's ``amount`` field contain a value less than $100.00?

For more on leveraging application data in an oso policy, check out
:doc:`/getting-started/policies/application-types`.

.. admonition:: What's next
    :class: tip whats-next

    * Explore how to :doc:`/getting-started/application/index`.
    * Dig deeper on :doc:`/getting-started/policies/index`.
    * Check out oso in action: :doc:`/using/examples/index`.
    * Explore the :doc:`/more/design-principles` behind oso.

------------------------

.. include:: /newsletter.rst

.. spelling::
   cURL
