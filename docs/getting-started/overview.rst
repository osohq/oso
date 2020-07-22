.. role:: polar(code)
   :language: prolog

============
Overview
============

oso helps developers build authorization into their applications. Authorization typically starts simple – perhaps a few `if` statements in your code – but can  grow complex as you add:

- More roles
- Dynamic permissions
- Hierarchies
- External identity data, e.g., from LDAP or OAuth
- Customer-configurable permissions

These can be hard to express concisely, and over time what started as a small number of simple `if` statements can become a large amount of custom authorization logic spread throughout a codebase, which can be hard to maintain, modify, debug, and secure.

oso is designed to solve these problems based on 3 principles, which we'll describe briefly then in more detail below.

1. **Separation of concerns.** Authorization logic is distinct from business logic. By separating the two, you can make changes to the policy which apply across the entire application, write reusable patterns, and get a single place to control, test and visualize access.
2. **Right tool for the job.** Authorization deals in facts and rules about who is allowed to do what in a system. Solutions to describe authorization logic ought to be declarative and have semantics that map to common domain concepts – like roles and relationships, or boolean conditions over the input attributes.
3. **Authorization decisions and application data are inseparable.** Authorization decisions always rely on application context – like who a user is and her relationship to the data she's trying to access. The authorization system ought to be able to call directly into the application so you can write policies using the application's objects and data.

What is oso
-----------

oso is an open source policy engine for authorization that's embedded in your application. It provides a declarative policy language for expressing authorization logic, which you define separately from your application code but which executes inside the application and can call directly into it.

Here are some of the nuts and bolts.

oso ships as a **library**.

.. container:: left-col

    .. code-block:: python
        :caption: :fab:`python` app.py

        if oso.query("allow", [user, "view", expense]):
            # ...

.. container:: right-col

    .. code-block:: polar
        :caption: :fa:`oso` expense.pol

        allow(user, "read", expense) if
            user.email = expense.submitted_by;

You write policies to define authorization logic. You tell it things like who should access what, based
on what you know about them and their relationship to what they are trying to access.
Policies are written in a declarative policy language called Polar, then they are loaded into oso.

You can use oso to query the policies and pass in relevant application data as inputs.

.. todo:: Wording

The oso policy engine answers queries by evaluating the policy, and calls into the application to
read attributes off of application data, to check types or to look up class methods.

oso also ships with a debugger and a REPL.

Let's return to our three principles in more detail.

.. _separation-of-concerns:

Separation of Concerns
----------------------

Let's imagine we're building a SaaS app that allows organizations to manage their
employees' expenses. We need authorization logic to ensure that, for example, employees can only view their own expenses, and to ensure that their managers can view and approve their team's expenses.

At first, having ``if`` statements to represent this logic is not that big a deal. But over time, across multiple
files and multiple parts of the application, we end up with pieces of authorization logic
sprinkled throughout our codebase. This creates a de facto permissions system that can be hard to keep track of
or change.

Splitting out authorization logic with oso might look as follows:

.. tabs::

    .. group-tab:: Before

        .. code-block:: python
           :caption: :fab:`python` expense.py

            def show(expense):
                if user.email == expense.submitted_by:
                    return str(expense)

            def download(expense):
                if user.email == expense.submitted_by:
                    return expense.to_json()

            def approve(expense):
                if any(employee.email == expense.submitted_by for employee in user.employees()):
                    expense.approve()

    .. group-tab:: After

        .. container:: left-col

            .. code-block:: python
                :caption: :fab:`python` expense.py

                def show(expense):
                    if oso.query("allow", [user, "read", expense]):
                        return str(expense)

                def download(expense):
                    if oso.query("allow", [user, "read", expense]):
                        return expense.to_json()

                def approve(expense):
                    if oso.query("allow", [user, "approve", expense]):
                        expense.approve()

        .. container:: right-col

            .. code-block:: polar
                :caption: :fa:`oso` expense.pol

                # employees can read expenses they submitted
                allow(user, "read", expense) if
                    submitted(user, expense);

                # managers can approve employee expenses
                allow(user, "approve", expense) if
                    employee in user.employees and
                    submitted(employee, expense);

                submitted(user, expense) if
                    user.email = expense.submitted_by;

.. note::
    Want to see how this policy works? Check out the :doc:`guide for writing policies </using/policies/index>`.

The ``oso.query`` call can be made anywhere. So even if we have developer APIs
and multiple different backend server calls -- which all require checking the
user's permissions for viewing an expense -- the actual logic is all in one place.

By taking this approach, the logic becomes more maintainable. For example, we can
extract out common patterns into reusable code. We can write a rule :polar:`submitted(user, expense) if user.email = expense.submitted_by`, which we then use in multiple places.
If we wanted to change this logic by instead looking up the user ID,
we only need to change this one line.

Similarly, creating or modifying permissions means making changes to just the policy file, and having them applied throughout the application. Meaning we are less likely
to either break a workflow by forgetting to update permissions somewhere, and less
likely to introduce a security hole.

For example, we ensure that if you can see an expense in the UI (the ``show`` method), then you can download it as JSON.
Any modifications to the ``allow`` rule for reading an expense will be consistent across the two of them.

If we need to extend the permissions to make ``download`` stricter,  we just add a rule that inherits from ``read`` and
adds more conditions: :polar:`allow(user, "download", expense) if allow(user, "read", expense) and user.has_mfa_enabled()`;

Furthermore, by conforming to a standardized approach to authorization, we can leverage
tooling built around the standard. For oso, this means access to :doc:`a policy debugger and interactive REPL </using/dev-tools/index>`.

Right tool for the job
----------------------

If you ask someone to describe the permissions a user should have in a system
using natural language, you will generally find they have no problem doing so.
What often happens, however, is the authorization system used makes it hard
to take an intuitive concept and implement it.

oso policies are written using a declarative language, designed specifically
for expressing authorization logic in applications. This means that you write what you want the outcome to be, and oso worries about things like the order in which to run operations, and how to achieve the desired end goal.

Let's take a slightly more complex example continuing from above. Suppose we now
have two different user types who can approve expenses. With oso, that might look like:

.. container:: left-col

    .. code-block:: polar
        :caption: :fa:`oso` expense.pol

        # managers can approve their employees' expenses
        allow(user, "approve", expense) if
            manages(user, employee)
            and submitted(employee, expense);

        # project managers can approve project expenses
        allow(user, "approve", expense) if
            role(user, "manager",
                Project.lookup_by_id(expense.project_id));

.. container:: right-col

    .. code-block:: polar
        :caption: :fa:`oso` organization.pol

        # manages user or managers users' manager
        manages(manager, user) if
            employee in manager.employees()
            and employee = user
            or manages(employee, user);

        # user is in the list of project managers
        role(user, "manager", project: Project) if
            user in project.managers();

.. tip::
    For full examples of the patterns used here, check out the following guides:

    - :ref:`abac-basics`
    - :ref:`abac-hierarchies`
    - :ref:`abac-rbac`

These two policies capture a lot of authorization logic, without sacrificing
ease of understanding. The *declarative* nature of this matches well with the
problem at hand: we are declaring new properties about our system – like what
it means to have submitted an expense or to manage someone – and then we combine
these into new statements that declare what users can do in the system.

The policy stays short and relatively flat because oso handles the evaluation.
You don't need to specify *how* to apply these rules. If we query oso using the above policy to see if a user can read an expense, oso will handle everything from determining which rules it needs
to apply, and their relative ordering, to calling into the host
application to lookup the email field on the user object. You give oso
all the ingredients, then oso searches through everything and puts them together in the necessary order to make a decision.

But in testing the application, we realize that managers can't even read
the expenses they are supposed to be approving! Instead of repeating all
the same logic from above, we can add some simple structure:

.. code-block:: polar

    allow(user, "read", expense) if
        allow(user, "approve", expense);

.. todo:: Is this getting a little too deep into examples? Also, conclusion wording.

This intuitively addresses the problem from before, and adds an entirely
new dimensions of permissions with just a single rule.


.. todo:: Should we link to the performance discussion and be frank with it
          as a shortcoming?


Authorization decisions and application data are inseparable
------------------------------------------------------------

Some applications may never need to go beyond basic role-based access control (RBAC).
Perhaps there are users and administrators, but otherwise all users are treated equally.
However, any application that needs to control access to data needs to
determine access based on *who* the user is and her *relation* to the data.

Looking through the examples we have on this page, we've accessed attributes like
email addresses, who submitted an expense, and referenced methods like managers
associated to a project, or employees associated to a manager.

All of these were core pieces of business logic, and won't be going away soon.
Depending on the implementation, one could easily imagine the ``employees()`` method
being handled by somewhere as a SQL join statement.

To use this information for authorization decisions means we either need to duplicate
this logic elsewhere, or leverage the exising business logic we already have access to.

At its best, authorization logic weaves together discrete bits of business logic into a
rich authorization tapestry. Striking a balance between using application data wherever
its needed to make decisions, while keeping the code clean, reusable, and maintainable.


.. admonition:: What's next?

    Stay here and continue reading about what lies under the hood of the oso library.

    Head back to :doc:`/getting-started/quickstart` if you
    haven't already, or continue on to :doc:`/using/auth-fundamentals`.


Internals
---------

.. todo::
    Move this to a different introductory section? Feels a bit misplaced here.

oso is supported in :doc:`a number of languages </reference/libraries/index>`, but the `core of oso <https://github.com/osohq/oso>`_ is written in Rust, with bindings for each specific language.

The core of oso is an implementation of the **Polar language**. This handles
parsing policy files, and executing queries in the form of a virtual machine.
oso was designed from the outset to be able to be natively embedded in different
languages. It exposes a foreign function interface (FFI) to allow the calling
language to drive the execution of its virtual machine.


.. todo::
    better wording for "in the form of a virtual machine"

oso can read files with the ``.pol`` suffix, which are policy files written in Polar syntax.
These are parsed and loaded into a *knowledge base*, which can be thought of an
in-memory cache of the rules in the file.

Applications using oso can tell it relevant information, for example registering
classes to be used with policies, which are similarly stored in the knowledge base.
The oso implementation can now be seen as a bridge between the policy code and the application classes.

The oso library is responsible for converting types between oso primitive types
(like strings, numbers, and lists), and native application types (e.g. Python's ``str``,
``int``, and ``list`` classes), as well as keeping track of instances of application classes. When executing a query like ``oso.query("allow", [user, "view", expense])`` oso creates a new virtual machine to execute the query. The virtual machine executes, returning to the native library whenever some application-specific information is needed.
