.. role:: polar(code)
   :language: prolog

=================
Design Principles
=================

oso helps developers build authorization into their applications.
Authorization typically starts simple – perhaps a few ``if`` statements
in your code – but can grow complex as you add:

- More roles
- Dynamic permissions
- Hierarchies
- External identity data, e.g., from LDAP or OAuth
- Customer-configurable permissions

These can be hard to express concisely, and over time what started as a small
number of simple ``if`` statements can become a large amount of custom
authorization logic spread throughout a codebase, which can be hard to maintain,
modify, debug, and secure.

oso is designed to solve these problems based on three principles, which we'll
describe briefly here, then in more detail below.

**1. Separation of concerns, but not data.** Authorization logic is distinct
from business logic, and by separating the two, you can make changes to your
policies that apply across your application, write reusable patterns, and get
one place to control, test and visualize access. But authorization decisions
always rely on application context, like who a user is and what their relationship
is to the data they're trying to access. So without being *part* of the
application, policy code should to be able to call into the application
and use application objects and data.

**2. Right tool for the job.** Authorization deals in facts and rules about who
is allowed to do what in a system. Solutions to describe authorization logic
ought to be declarative and have semantics that map to common domain concepts –
like roles and relationships, or boolean conditions over the input attributes.

**3. Easy to start, powerful when you need it.** No two
authorization problems are the same, because no two applications are the same.
And so while many authorization problems can be made to fit a general pattern
like roles, the model's fit typically degrades as you add more – and more
complex – requirements. An authorization system should provide simple,
opinionated building blocks to start but should not force developers to bend
their requirements to the capabilities of the system. Instead, it should give
them the ability to extend the system to solve the use case at hand.

Separation of concerns, but not data
------------------------------------

Let's imagine we're building a SaaS app that allows organizations to manage
their employees' expenses, and that our authorization policy needs to express
(at least) the following logic:

* Employees can only view expenses they submitted.
* A manager can view and approve their team's expenses.

We might start by embedding this logic directly into the relevant
application methods, e.g.

.. code-block:: python
   :caption: :fab:`python` simple authorization logic

    if user.email == expense.submitted_by:
        ...

But then our policy logic is intertwined with application and business
logic, and diffuses throughout the application. Policy changes, audits,
etc. become complicated ordeals, because there is no single place where
"the policy" lives.

The maintainable solution to this problem is to factor out the
authorization logic from that of the application, and make a single,
uniform call when we need to authorize a request. Here's what that
might look like with oso:

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
                    if oso.is_allowed(user, "read", expense):
                        return str(expense)

                def download(expense):
                    if oso.is_allowed(user, "read", expense):
                        return expense.to_json()

                def approve(expense):
                    if oso.is_allowed(user, "approve", expense):
                        expense.approve()

        .. container:: right-col

            .. code-block:: polar
                :caption: :fa:`oso` expense.polar

                # employees can read expenses they submitted
                allow(user, "read", expense) if
                    submitted(user, expense);

                # managers can approve employee expenses
                allow(user, "approve", expense) if
                    employee in user.employees() and
                    submitted(employee, expense);

                submitted(user, expense) if
                    user.email = expense.submitted_by;

.. tip::
    Want to see how this policy works? Check out the
    :doc:`guide for writing policies </getting-started/policies/index>`.

In this example, we've factored out the authorization logic into an
oso policy file, and inserted calls to ``oso.is_allowed`` in its place.
All of the actual logic now resides in oso, which means that changing
permissions, auditing, etc. can all happen in one place.

The key thing we did *not* do, however, was to separate the authorization
logic from the objects it is *about*. Because oso operates as a library
embedded within your application, it has direct access to application
data, objects, and methods. For instance, in the last line of the policy
above, the term :polar:`expense.submitted_by` means just what you'd think:
it looks up the ``submitted_by`` attribute on the ``expense`` object,
and returns the value of that field. But the ``expense`` object is passed
directly into oso from your application; it "lives" in the application.
If that attribute happened to name a method instead of a field, it would
be called (with no arguments) *within your application's runtime context*,
and the result passed back to oso. Thus, oso can use your application's
native objects to make its authorization decisions, while at the same time
keeping authorization logic separate from application logic.


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
        :caption: :fa:`oso` expense.polar

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
        :caption: :fa:`oso` organization.polar

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
You don't need to specify *how* to apply these rules. If we query oso using the
above policy to see if a user can read an expense, oso will handle everything
from determining which rules it needs to apply, and their relative ordering, to
calling into the host application to lookup the email field on the user object.
You give oso all the ingredients, then oso searches through everything and puts
them together in the necessary order to make a decision.

Easy to start, powerful when you need it
----------------------------------------

Some applications may never need to go beyond basic role-based access control
(RBAC). Perhaps users belong to organizations, and all users fit into one of
several roles. So most access can be reduced to checking the user has the right
role for the URL they are accessing.

So most authorization can be reduced to something simple like:

.. code-block:: polar
    :caption: :fa:`oso`

    allow(user, action, path) if
        user.role = "admin";

    allow(user, action, path) if
        user.role = "user"
        and not path.starts_with("/admin/");

But for many applications, over time, new features get added that don't quite
fit the same model. Maybe a user can now belong to multiple organizations, so
you need to check whether they are the specific role for the organization they
are accessing. Or maybe a user can have their own private data separate to the
data shared with the rest of their organization. Perhaps support staff need to
have full access to a limited amount of data, so an exception case is made.

And in time, the number of roles grows... The number of different permissions
combined with the number of roles leads to an exploding number of combinations.

The goal of oso is to make easy things *easy*, and hard things *possible*.
In our various :doc:`examples </using/examples/index>`, we take you from
:doc:`simple roles </using/examples/rbac>`, up to more complex versions.
Combining :doc:`roles with attributes </using/examples/abac>`,
applying :doc:`inheritance structure </using/examples/inheritance>` and
:ref:`hierarchies <abac-hierarchies>`. Each of these guides building
up from a simple access model, and then adding in complexity
while leaning on the data that already exists in your system.

And this authorization logic can be woven in throughout the application.
In our :doc:`guide to adding oso </getting-started/application/index>`
we show how to do authorization at the API layer, and in the controller code.
But if that doesn't match your needs, underneath all of this is a system
that is powerful to adapt to :doc:`many other patterns </getting-started/application/patterns>`.

Overall, our answer for how to best use oso is dependent on what's best
for you.

.. admonition:: What's next?
    :class: tip whats-next

    Head back to :doc:`/getting-started/quickstart` if you
    haven't already, or continue on to :doc:`/more/key-concepts`.
