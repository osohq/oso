.. role:: polar(code)
   :language: prolog

============
Overview
============

oso helps developers build authorization into their applications. Authorization typically starts simple, but can quickly grow complex as you add:

- More roles
- Dynamic permissions
- Hierarchies
- External identity data, e.g., from LDAP or OAuth
- Customer-configurable permissions

Authorization logic involving these is typically hard to express concisely, and over time tends to spread throughout a codebase, making it hard to debug, maintain, and secure. oso lets you put all of your authorization logic in one place, expressed in a policy language that seamlessly integrates your application's data.

oso is designed based on 3 principles, which we'll describe briefly here then in more detail below.

1. **Separation of concerns.** Authorization logic is distinct from business logic. By separating the two, you can make changes to the policy which apply across the entire application, write reusable patterns, and get a single place to control, test and visualize access.
2. **Right tool for the right job.** Authorization deals in facts and rules about who is allowed to do what in a system. Solutions to describe authorization logic ought to be declarative and have semantics that map to common domain concepts – like roles and relationships, or boolean conditions over the input attributes.
3. **Authorization decisions and application data are inseparable.** Authorization decisions always rely on application context – like who a user is and her relationship to the data she's trying to access. The authorization system ought to be able to call directly into the application so you can write policies using the application's objects and data.

Key Pieces
----------

Before we go into more depth about the principles, it might be helpful to
go over the key pieces of oso. If you're already familiar with oso,
feel free to :ref:`jump to the next section <separation-of-concerns>`.

First of all, oso is a **library** for evaluating authorization policies in your application.

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

You write policies to encapsulate authorization logic. You tell it things like who should access what, based
on what you know about them and their relationship to what they are trying to access.
Policies are written in a declarative policy language called Polar, and then loaded into oso.

You can use oso to query the policies, passing in the relevant application data as inputs.

.. todo:: Wording

The oso policy engine answers queries by evaluating the policy, and will call into the application in order to
read attributes off of application data, to check types or look up class methods.

Now let's return to our three principles in more detail.

.. _separation-of-concerns:

Separation of Concerns
----------------------

Let's imagine we're building a SaaS app that allows organizations to manage their
employee expenses. We'll need authorization logic to restrict access to, for example, allow employees to view their own expenses, and their managers to view and approve their expenses.

At first, having ``if`` statements is not that big a deal. But over time, across multiple
files and multiple parts of the application, you end up with pieces of authorization logic
sprinkled throughout your codebase. This creates an defacto permissions system that is hard to update
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
    Want to see how this policy works? Check out the :doc:`guide for writing policies </understand/policies/index>`.

The ``oso.query`` call can be made anywhere. So even if we have developer APIs
and multiple different backend server calls -- which all require checking the
user's permissions for viewing an expense -- the actual logic is all in one place.

By taking this approach, the logic becomes more maintainable. For example, we can
extract out common patterns into reusable code. We can write a rule :polar:`submitted(user, expense) if user.email = expense.submitted_by`, which we then use in multiple places.
If we wanted to change this logic by instead looking up the user ID,
we only need to change this one line.

Similarly, creating or modifying permissions means making changes to just the policy file, and having them applied throughout the application. Meaning you are less likely
to either break a workflow by forgetting to update permissions somewhere, and less
likely to introduce a security hole.

For example, we ensure that if you can see an expense in the UI (the ``show`` method), then you can download it as JSON.
Any modifications to the ``allow`` rule for reading an expense will be consistent across the two of them.

If you need to extend the permissions to make ``download`` stricter? Just add a rule which inherits from ``read`` and
adds more conditions: :polar:`allow(user, "download", expense) if allow(user, "read", expense) and user.has_mfa_enabled()`;

Furthermore, by conforming to a standardized approach to authorization, you can leverage
tooling built around the standard. For oso, this means access to :doc:`a policy debugger and interactive REPL </reference/dev-tools/index>`.

Right tool for the job
----------------------

If you ask someone to describe the permissions a user should have in a system
using natural language, you will generally find they have no problem doing so.
What often happens, however, is the authorization system used makes it hard
to take an intuitive concept and implement it.

oso policies are written using a declarative language, designed specifically
for writing authorization logic in applications. This means that you write what you want the outcome to be, and oso worries about things like what order to run things in, and how to achieve the desired end goal.

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
problem at hand: we are declaring new properties about our system, like what
it means to have submitted an expense, or to manage someone, and then combine
these into new statements declaring what users can do in the system.

The policy stays short and relatively flat because the evaluation is handled by oso.
You don't need to specify *how* these rules should be applied. You give it
all the ingredients and it searches through everything you tell it
, and puts them together in the necessary order to make decisions.

If we query oso using the above policy to see if a user can read an expense
or not, oso will be handling everything from determining which rules need
to be applied, and the relative ordering of them, to calling into the host
application to lookup the email field on the user object.

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

Many applications never need to go beyond basic role-based access control (RBAC).
Perhaps there are users and administrators, but otherwise all users are treated equally.
However, any application which needs to control access to data will need to venture into
determining access based on *who* the user is and their *relation* to the data.

That's why the policies you see on this page are all about those attributes - 
"does the user's email match the email that submitted the expense?", "is the user
a manager of the project the expense?".

But all of this data is likewise core to the application, so there is no possibility of
extricating it. Leaving us with the following options:

.. todo:: Examples of these? How can we make these more concrete?

* Build authorization as a separate consumer of the same application data.

  * Now we have another system to keep in sync with potentially every other application, and possibly duplicate all of the classes and methods used to access data. 

* Synchronize relevant data into the authorization system.

  *  How frequently should this be done? What data will be needed in the new system?

* Leave authorization to the application.

  *  Starting to sound pretty good right about now.


At its best, authorization logic weaves together discrete bits of business logic into a
rich authorization tapestry. Striking a balance between using application data wherever
its needed to make decisions, while keeping the code clean, reusable, and maintanable.


.. admonition:: What's next?

    Stay here and continue reading about what lies under the hood of the oso library.

    Head back to :doc:`/getting-started/quickstart` if you
    haven't already, or continue on to :doc:`/understand/auth-fundamentals`.


.. todo::
    Move this to a different introductory section? Feels a bit misplaced here.

Internals
---------

oso is supported in :doc:`various languages </reference/libraries/index>`, but the `core of oso <https://github.com/osohq/oso>`_ is written in Rust, with bindings for each specific language. This library is designed to make it easy to add complex authorization to any application.

The core of oso is an implementation of the **Polar language**. This handles
parsing policy files, and executing queries in the form of a virtual machine.
oso was designed from the outset to be able to be natively embedded in different
languages, and so the foreign function interface (FFI) exposed allows the calling
language to drive the execution of the virtual machine.


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
