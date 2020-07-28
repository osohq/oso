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

**3. Freedom to extend.** No two authorization problems are the same,
because no two applications are the same. And so while many authorization
problems can be made to fit a general pattern like roles, the model's fit
typically degrades as you add more – and more complex – requirements.
An authorization system should provide simple, opinionated building blocks
to start but should not force developers to bend their requirements to the
capabilities of the system. Instead, it should give them the ability to
extend the system to solve the use case at hand.

Separation of Concerns, but not Data
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


Right Tool for the Job
----------------------

If you ask someone to describe the permissions a user should have in a system
using natural language, you will generally find they have no problem doing so.
What often happens, however, is that authorization systems make it hard to
take an intuitive concept and implement it as a concrete security policy.

oso policies are written using a declarative language designed specifically
for expressing authorization logic in applications. This means that you write
permissions as simple logical statements, and oso performs the necessary
inferences to go from what you have (application objects and information
about the request you're trying to authorize) to a yes/no authorization
decision. Rule ordering, access to application objects, and other such
ancillary tasks are handled transparently by the system.

Let's illustrate this by continuing our example from above.
Suppose that we now have two different user types who can approve expenses:
direct managers, and project managers. With oso, that might look like this:

.. container:: left-col

    .. code-block:: polar
        :caption: :fa:`oso` expense.polar

        # managers can approve their employees' expenses
        allow(user, "approve", expense) if
            manages(user, employee)
            and submitted(employee, expense);

        # project managers can approve project expenses
        allow(user, "approve", expense) if
            role(user, "manager", Project.lookup_by_id(expense.project_id));

.. container:: right-col

    .. code-block:: polar
        :caption: :fa:`oso` organization.polar

        # manages user or manages users' manager
        manages(manager, user) if
            employee in manager.employees()
            and employee = user
            or manages(employee, user);

        # user is in the list of project managers
        role(user, "manager", project: Project) if
            user in project.managers();

.. tip::
    For full examples of the patterns used here, see the following guides:

    - :ref:`abac-basics`
    - :ref:`abac-hierarchies`
    - :ref:`abac-rbac`

The policy stays short and relatively flat because oso handles the evaluation.
You don't need to specify *how* to apply these rules. If we query oso using the
above policy to see if a user can read an expense, oso will handle everything
from determining which rules it needs to apply and their relative ordering, to
calling into the host application to lookup the email field on the user object.
You give oso all the ingredients, then oso searches through everything and puts
them together in the necessary order to make a decision.

Freedom to Extend
-----------------

Some applications may never need to go beyond basic role-based access control
(RBAC). You can :doc:`express that in oso easily </using/examples/rbac>`.
And likewise :doc:`ABAC </using/examples/abac>`,
and :doc:`inheritance </using/examples/inheritance>`, etc.
oso is purposefully agnostic to the *kind* of authorization logic
that you need; its job is to make expressing simple policies easy,
and complex policies possible.

Because the oso policy engine is an interpreter for a Turing-complete
domain specific language, it is not limited to a fixed set of configuration
parameters, or prescribed authorization structures. And because it offers
direct integration with your application's data and methods, it is not
limited to just the data you choose to "package up" for it and ship
across a wire, nor does it force you to duplicate application logic
in policy code. Instead, it acts as an *extension of your application*
that encapsulates, but does not limit, your authorization logic.

.. kill this paragraph?

As we developed oso, we talked to a lot of organizations with a lot
of different kinds of authorization requirements. Internally-facing,
customer-facing, subject to stringent regulations, dependent on data
that lives in a foreign system, etc. Endless variations. Most of the
ones with even moderately complex requirements ended up investing
heavily in custom code and frameworks, either up front, before the
complexity exploded (rare) or after the fact (much more common, and
much more costly).

oso helps you tame complex authorization problems by *abstraction*
and *extension*. By abstracting away from, and yet fully supporting:

* specific application languages and frameworks
* specific authorization schemes
* rigid network-based interfaces

You can adapt oso to meet even the most complex authorization requirements,
because you extend the built-in system to encapsulate them, and then
embed the whole engine in your application -- extending your application --
so that it can make decisions that are intrinsically coupled to the data
and behaviors that reside there.

.. admonition:: What's next?
    :class: tip whats-next

    Head back to :doc:`/getting-started/quickstart` if you
    haven't already, or continue on to :doc:`/more/glossary`.
