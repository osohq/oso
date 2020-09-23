==========
Quickstart
==========

oso helps developers build authorization into their applications. If you've
never used oso before and want to see it in action, this guide is for you.

In general, it takes less than 5 minutes to add oso to an existing application
and begin writing an authorization policy.

In this guide, we're going to walk through how to add authorization to a simple web server using oso.
You'll be able to follow along by editing the sample application in the repl.it environment below.

The source code of the completed application is also available on `GitHub <TODO LINK>`_.

Expenses Application
====================

Our sample application serves data about expenses submitted by users.

If you navigate to ``quickstart/expenses.py``, you'll see a simple ``Expense`` class, and some data stored in the
``EXPENSES`` dictionary.

The HTTP server code is stored in ``quickstart/server.py``, where we have defined a route handler for ``GET`` requests to
the path ``/expenses/{id}``.

You can run the server by hitting the "Run" button while on ``server.py``, or from the command line with:

.. code-block:: text

  $ poetry run python quickstart/server.py

The app doesn't have any authorization yet, so if you head to localhost:5050/expenses/1, for example,
you'll see the first expense displayed.

.. TODO: explain the target auth scheme here (e.g. you can view an expense if you submitted it or whatever)

Now we can add oso to control who has access to the expenses data.

Adding oso
==========

Adding oso to an application basically consists of three steps:

1. Create a .polar policy file
2. Initialize the global oso instance by loading the policy and registering relevant application classes
3. Add calls to Oso.is_allowed() at authorization enforcement points

The repl.it environment should already have oso installed from our ``poetry.lock`` files. To use oso locally,
you can find download and installation instructions :doc:`here </download>`.

Creating a policy
-----------------

oso policies are written in a declarative policy language called Polar. Polar files have the ``.polar`` extension.
We've already created a policy file for this project, ``expenses.polar``, which you can find in the project's root
directory. oso policies are made up of :ref:`Polar rules <TODO RULES LINK>`. You can include any kind of rule in a policy,
but the oso library is designed to evaluate :ref:`allow rules <TODO>`, which specify the conditions that allow an
**actor** to perform an **action** on a **resource**.

We'll leave the policy empty for now, but we'll come back to it and add rules later.

Initializing the oso instance
-----------------------------

To use the oso library, we need to create a global instance of the ``Oso`` class, which we will use to get
authorization decisions from oso. We also need to load our policy file by calling ``Oso.load()``, and register
any application classes that we want oso to know about using ``Oso.register_class()``.
We're going to register the ``Expense`` class now, since we'll use it in our policy later on.

Initialize the ``Oso`` instance by adding the following lines of code
after the imports in ``quickstart/server.py``:

.. code-block:: python

  from oso import Oso

  oso = Oso()
  oso.register_class(Expense)
  oso.load_file("expenses.polar")

Enforcing authorization
-----------------------

The ``Oso`` instance exposes a method to evaluate ``allow`` rules that takes three
arguments, **actor**, **action**, and **resource**:

.. literalinclude:: /examples/quickstart/python/allow-01.py
    :language: python
    :lines: 12-14

The above method call returns ``true`` if the **actor** ``"alice@example.com"`` may
perform the **action** ``"GET"`` on the
**resource** ``EXPENSES[1]``. We're using ``"GET"`` here to match up with the HTTP
verb used in our server, but this could be anything.

.. note:: For more on **actors**, **actions**, and **resources**, check out
  :doc:`/more/glossary`.

We want to call this method at our application's authorization enforcement points. In this app,
we'll enforce authorization for our expense data in the ``/expenses/{id}`` route handler.

As in the example above, the **actor** will be the authenticated user's email address.
In lieu of setting up real identity and authentication systems, we'll use a
custom HTTP header to indicate that a request is "authenticated" as a particular
user. We'll use the HTTP request method, in this case ``"GET"`` as the **action**, and
the **resource** is the expense retrieved from our stored expenses.


Update the ``do_GET()`` method in ``quickstart/server.py`` so that it looks like this:

.. code-block:: python

    def do_GET(self):
        try:
            _, resource, id = self.path.split("/")
            if resource != "expenses":
                return self._respond("Not Found!", 404)
            expense = db[int(id)]
        except (ValueError, KeyError):
            return self._respond("Not Found!", 404)

        actor = self.headers.get("user", None)
        action = "GET"
        if oso.is_allowed(actor, action, expense):
            self._respond(expense)
        else:
            self._respond("Not Authorized!", 403)


oso's authorization system is deny-by-default. Since we haven't yet written any
policy code, no one is allowed to view any expenses. To see that in action,
start the server and try to ``"GET"`` an expense with the following curl command:

.. code-block:: console

    TODO

Now that we have all our authorization plumbing in place, we can implement our permissions scheme
by writing some rules.

Adding our first rule
=====================

In our policy file (``quickstart/expenses.polar``), let's add a rule that allows anyone with
an email ending in ``"@example.com"`` to view all expenses. That way, everyone at Example.com, Inc. will be
able to view expenses, but no one outside the company will be able to.

Add the following rule to ``quickstart/expenses.polar``:

.. literalinclude:: /examples/quickstart/polar/expenses-03-py.polar
    :language: polar
    :caption: :fa:`oso` expenses.polar
    :class: copybutton

.. |str_endswith| replace:: the ``str.endswith`` method
.. _str_endswith: https://docs.python.org/3/library/stdtypes.html#str.endswith

If you noticed that the ``.endswith`` call looks pretty familiar, you're
right on --- oso is actually calling out to |str_endswith|_ defined in the
Python standard library. The **actor** value passed to oso is a Python
string, and oso allows us to call any ``str`` method from Python's standard
library on it.


Once we've added our new rule and restarted the web server, every
user with an ``@example.com`` email should be allowed to view any expense:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(...)

.. TODO: decide if we still need the following three paragraphs

Okay, so what just happened?

When we ask oso for a policy decision via ``Oso.is_allowed()``, the oso engine
searches through its knowledge base to determine whether the provided
**actor**, **action**, and **resource** satisfy any **allow** rules.

In the above case, we passed in ``alice@example.com`` as the **actor**, ``"GET"`` as the
**action**, and ``EXPENSE[1]`` as the **resource**, satisfying our rule that allows
anyone with an email ending in ``"@example.com"`` to view any expense.

If a user's email doesn't end in ``"@example.com"``, the rule fails, and they
are denied access:

.. code-block:: console

  $ curl -H "user: alice@foo.com" localhost:5050/expenses/1
  Not Authorized!

.. note::
  Each time you load a file, it will load the policy
  **without** clearing previously loaded rules. Be sure to
  clear oso using the ``clear_rules()`` method if you want to
  invalidate previously loaded rules.

If you aren't seeing the same thing, make sure you created your policy
correctly in ``expenses.polar``.

A Quick Note on Type Checking
-----------------------------
.. TODO cut this down and include the String specializer; maybe this doesn't go here?

You may have already guessed that the ``Expense`` term following the colon in the head of our policy rule
specifies a parameter type restriction. This is a :ref:`specializer <Specialization>`, a pattern that controls rule
execution based on whether the supplied argument matches it. Here, we specialize the third argument on
our own ``Expense`` class, which will restrict this rule to arguments that are instances of that class or any
subclass. Specializers are optional but highly recommended to avoid bugs that could arise if
an unexpected type of resource is passed into a certain rule. We'll see more examples of specializers later in this guide.


Writing Authorization Policy Over Application Data
==================================================

We now have some basic access control, but the higher-ups at Example.com, Inc. aren't satisfied with
our policy that allows all employees to see each other's expenses. They
would like us to modify the policy such that employees can only see their own
expenses.

To accomplish that, we can **replace** our existing rule with:

.. literalinclude:: /examples/quickstart/polar/expenses-04.polar
    :language: polar
    :caption: :fa:`oso` expenses.polar
    :class: copybutton

Behind the scenes, oso looks up the ``submitted_by`` field on the provided
``Expense`` instance and compares that value against the provided **actor**.
And just like that, an actor can only see an expense if they submitted the expense.

Now Alice can see her own expenses but not Bhavik's:

.. code-block:: console

  TODO: update links

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(...)
  $ curl -H "user: alice@example.com" localhost:5050/expenses/3
  Not Authorized!

And vice-versa:

.. code-block:: console

  TODO: update links

  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Not Authorized!
  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/3
  Expense(...)

We encourage you to play around with the current policy and experiment with
adding your own rules!

For example, if you have ``Expense`` and ``User`` classes defined in your
application, you could write a policy rule in oso that says a ``User`` may
approve an ``Expense`` if they manage the ``User`` who submitted the expense
and the expense's amount is less than $100.00:


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
- Is their manager the approver?
- Does the expense's ``amount`` field contain a value less than $100.00?

.. note:: For more on leveraging application data in an oso policy, check out
  :doc:`/getting-started/policies/application-types`.



Summary
=======

We just went through a ton of stuff:

* Installing oso.
* Setting up our app to enforce the policy decisions made by oso.
* Writing authorization rules over static and dynamic application data.

.. admonition:: What's next
    :class: tip whats-next

    * Explore how to :doc:`/getting-started/application/index`.
    * Dig deeper on :doc:`/getting-started/policies/index`.
    * Check out oso in action: :doc:`/using/examples/index`.
    * Explore the :doc:`/more/design-principles` behind oso.

------------------------

.. include:: /newsletter.rst

.. spelling::
    Gradle
