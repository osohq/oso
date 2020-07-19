.. Introduce some of the core oso concepts like rules/predicates + search
   semantics

========
Why oso?
========

Having gone through the :doc:`getting-started` guide, you may be
wondering what all the fuss is about. Why are we introducing a new language just
for the sake of replacing a few `if` statements in your code?

Or, you've been burned before by the promises of policy languages, and want to
know how oso is different.

In short, oso is `unique` because it is:

- a logic programming language (polar), which -- as we will see in this section and in `authorization models <auth-models>`_  -- makes it a great fit for writing policy as code
- embedded in your application, and can express authorization directly over your existing application data

Putting the two together: oso works as a natural extension of your application,
meaning you never have to write authorization logic sprinkled throughout your
application ever again.

Core concepts
-------------

polar is a **declarative language**. This means that
you write what you want the outcome to be, and the oso interpreter worries about
things like what order to run things in, and how to achieve the desired end
goal.

Other examples of declarative languages include SQL, and regular expressions. In
a SQL query, you don't dictate *how* the database should index, filter, and
aggregate the data, you ask it to return data fulfilling certain criteria. In
oso, it is the same.

However, one key difference is that polar is a **logic programming language**,
and based on decades and decades of prior work on Prolog. In logic programming,
your program is written in terms of statements which you assert are true. The
execution of the code then consists of asking it questions, of the form "given
what I told you, is this question true?".

Take a very simple example from the getting started guide. We ended up with two
`allow` rules:

.. code-block:: polar

    allow(actor, "read", "budget") if role(actor, "guest");
    allow(actor, "write", "budget") if actor.is_superuser;

We are asserting that an actor who is a superuser is allowed to "read" a "budget" resource, and that an admin actor is allowed to "write" a "budget".  

But when we ask a question like "is Alice allowed to read the budget?", we
don't impose how oso determines this. Instead oso **searches** through the
information it knows in order to **infer** whether the statement is true.

And we can actually ask oso much broader questions. In the previous query, we specified all
inputs to ask a specific question. But we can also ask questions like "what is
Alice allowed to do?". In oso, this query would be written as ``alice = new User
{ name: "alice" }, allow(alice, action, resource)``.

.. todo::
    if we're going to do this would need examples of how this actually
    works. This is the "Explain why logic programming is powerful" task

Because ``action`` and ``resource`` are both "unbound variables", meaning they do
not have a value assigned, oso will search for any values that make this true.
If Alice were a superuser and an admin, the query would return the set of results ``{action: "read", resource: "budget"}, {action: "write", resource: "budget"}``.

There are a couple of extremely powerful things happening here, with this
combination of **searching** and **inferences**.

Searching
---------

The execution of a polar query consists of searching through the rules you have
provided to find the right combination of rules and inputs to answer the
query.

Using the two simple rules defined above, to answer the above
query the search algorith may need to perform some number of steps like:

- Find all rules with name ``allow`` that expect three inputs
- Determine which order to apply those rules. The two rules above have the same types of inputs, so they are evaluated in the order in which they were defined.
- For each input, check that the input types match, e.g. if the input is ``action="write"``, this matches the second, not the first rule
- For each condition in the body, evaluate if they are true, e.g. check whether the user is indeed a superuser
- Evaluate any new queries nested within the rule, e.g. make a new query to determine if the actor is a guest with ``role(actor, "guest")``
- Any of the above steps may have produced multiple outcomes. If so, go back and try
  the next alternative.

In imperative programming, the programmer will be performing the equivalent
of these steps manually, typically with nested if statements:

.. code-block:: python

    if action == "read":
        if not user.super_user:
            return Unauthorized()
    elif action == "write":
        if user.role() != "admin":
            return Unauthorized()

Even in this simple case this logic is quite hard to follow. We would
need to repeat this throughout the application wherever we want to apply it,
and so small changes might leave you on the hook for making large refactors
throughout the codebase.

On the other hand, with oso you can make simple changes to the policy without
needing to touch your application code.

Inferences
-----------

One of the core abilities of logic programming is making *inferences*.
It can infer new conditions or properties based on what you already told it.

Continuing the simple example from before. Suppose we also have an "admin" role.
We might want admins to do anything that guests can do, so we write:

.. code-block:: polar

    role(actor, "guest") if role(actor, "admin");

This says that you can have the "guest" role if you already have the "admin" role.
With this rule, combined with the earlier rule stating
that guests can read budgets, oso infers that admins can also read
budgets.

A way to think of inferences is "you get out more than what you put in".
Every rule that you add gives oso more possible options and combinations
of things to try. Your work scales linearly, but the logic you can express
grows exponentionally -- this is some of the value added by the oso search algorithm.

To learn more about how polar and logic programming works head over to
:doc:`/language/polar-fundamentals`

oso in your application
-------------------------

So far we've seen what makes logic programming powerful. However until now logic
programming has only been available in the form of standalone languages, and
where they support some form of FFI, these are usually deeply entertwined with
the internals of the language.

What makes oso truly unique, is that it is embeddable in your application
as a simple library, and integrates directly with your application data.

What this means is that you can use oso as a natural extension of your app,
build up reusable logic in your policy and leverage it throughout your application.

.. todo::
    Add link to example app

Take the following snippet from the `example expenses app <#>`_:

.. code-block:: python

    from oso import polar_class, Oso

    @polar_class(from_polar="by_name")
    class User:
        """User model"""

        def __init__(self, name="", role="", location=""):
            # .. snip ..

        @classmethod
        def by_name(cls, name=""):
            """Lookup method to get a `User` object from the string name"""
            # .. snip ..

    @polar_class(from_polar="by_id")
    class Expense:
        """Expense model"""

        def __init__(self, amount: int, submitted_by: str, location: str, project_id: int):
            # .. snip ..

        @classmethod
        def by_id(cls, id: int):
            # .. snip ..

We can add the follow lines of Python:

.. code-block:: python

    oso = Oso()

    oso.load_str("owner(user: User, expense: Expense) if expense.submitted_by = user.name;")

    user = User.by_id(1)

    # not their expense
    expense = Expense.by_id(1)
    assert not oso.query("owner", user, expense)

    # is their expense
    expense = Expense.by_id(2)
    assert oso.query("owner", user, expense)

    oso.load_str("allow(user: User, action, expense: Expense) if owner(user, expense);")

    # user can read their own expense
    assert oso.allow(user, "read", expense)

In one policy line, we have defined the concept of data owners, using the existing
fields we have in our application. In a second policy line, we have expressed that
expense owners can interact with their own expenses.

Both of these lines are completely adaptable to other authorization models,
data structures, application structures, and so on.

.. todo::
    Maybe add Gabe's filesystem guide here as an in-depth version of the above?


.. pull-quote::
   **By integrating so deeply with the application domain, we are able to stand
   on the shoulders of the existing model relationships, weaving together
   discrete bits of business logic into a rich authorization tapestry.**

And this is just the beginning. Continue on to :doc:`/auth-models/index`
to see more examples of authorization models and how to implement those using oso.
