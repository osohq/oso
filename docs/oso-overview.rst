============
Overview
============

.. todo::
    "what is oso" wording 

oso is a policy engine for declarative authorization policies that executes and interacts directly in/with your application

Let's imagine we're building a SaaS app that allows organizations to manage their
employee expenses. We'll need authorization logic to restrict access to, for example, allow employees to view their own expenses, and their managers to view and approve their expenses.


oso is designed based on 3 principles, which we'll describe briefly then in more detail below

1. **Separation of concerns.** Authorization logic is distinct from business logic. By separating the two, you can make changes to the policy which apply across the entire application, write reusable patterns, and get a single place to control, test and visualize access.
2. **Right tool for the right job.** Authorization deals in facts and rules about who is allowed to do what in a system. Solutions to describe authorization logic ought to be declarative and have semantics that map to common domain concepts – like roles and relationships, or whether a policy is satisfied given certain inputs.
3. **Authorization decisions and application data are inseparable.** Authorization decisions always rely on application context – like who a user is and her relationship to the data she's trying to access. The authorization system ought to be able to call directly into the application so you can write policies using applications objects and data directly.

Key Pieces
----------

Before we go into more depth about the principles, it might be helpful to
give an overview of the key pieces of oso. If you're already familiar with oso,
feel free to :ref:`jump to the next section <separation-of-concerns>`.


First of all, oso is an **application framework for authorization** and is distributed
in the form of a **library**. oso is supported in :doc:`various languages </application-library/index>`, but the `core of oso <https://github.com/osohq/oso>`_ is written in Rust, with bindings for each specific language. This library is designed to make it easy to add complex authorization to any application.


The core of oso is an implementation of the **polar language**. This handles
parsing policy files, and executing queries in the form of a virtual machine.
oso was designed from the outset to be able to be natively embedded in different
languages, and so the foreign function interface (FFI) exposed allows the calling
language to drive the execution of the virtual machine.


.. todo::
    better wording for "in the form of a virtual machine"

.. todo::
    Is this too much? I thought it might be useful to provide some concrete information.

oso can read files with the ``.pol`` suffix, which are policy files written in polar syntax.
These are parsed and loaded into a *knowledge base*, which can be thought of an
in-memory cache of the rules in the file.

Applications using oso can tell it relevant information, for example registering
classes to be used with policies, which are similarly stored in the knowledge base.
The oso implementation can now be seen as a bridge between the policy code and the application classes.

The oso library is responsible for converting types between oso primitive types
(like strings, numbers, and lists), and native application types (e.g. Python's ``str``,
``int``, and ``list`` classes), as well as keeping track of instances of application classes. When executing a query like ``oso.allow(user, "view", expense)`` oso creates a new virtual machine to execute the query. The virtual machine executes, returning to the native library whenever some application-specific information is needed.


Now let's return to our three principles in more detail.

.. _separation-of-concerns:

Separation of Concerns
----------------------

With oso you separate authorization logic from your app by making a generic
``allow`` check using the library:

.. code-block:: python

    if oso.allow(user, "view", expense):
        # ...

and expressing the logic in the policy file:

.. code-block:: polar

    allow(user, "view", expense) if
        user.email = expense.submitted_by;

The ``oso.allow`` call can be made anywhere. So even if we have developer APIs,
and multiple different backend server calls which all require checking the
user's permissions for viewing an expense, the actual logic is all in one place.

By taking this approach, the logic becomes more maintainable. For example, we can
extract out common patterns into reusable code. We can write a rule ``submitted(user, expense) if user.email = expense.submitted_by``, which we then use in multiple places.
If we wanted to change this logic by instead looking up the user ID,
we only need to change this one line.

Similarly, creating or modifying permissions means making changes to just the policy file, and having them applied throughout the application. Meaning you are less likely
to either break a workflow by forgetting to update permissions somewhere, and less
likely to introduce a security hole.

Furthermore, by conforming to a standardized approach to authorization, you can leverage
tooling built around the standard. For oso, this means access to :doc:`a policy debugger and interactive REPL </dev-tools/index>`.

Right tool for the job
----------------------

If you ask someone to describe the permissions a user should have in a system
using natural language, you will generally find they have no problem doing so.
What often happens, however, is the authorization system used makes it hard
to take an intuitive concept and implement it.

oso policies are written usen a declarative language, designed specifically
for writing authorization logic in applications. This means that you write what you want the outcome to be, and oso worries about things like what order to run things in, and how to achieve the desired end goal.

Let's take a slightly more complex example continuing from above. Suppose we now
have three different people who can view expenses:

- Employees can view their own expenses
- Managers can view their employee's expenses
- Project managers can view expenses related to that project

With oso, that might look as follows:

.. code-block:: polar

    allow(user, "view", expense) if
        submitted(user, expense);

    allow(user, "view", expense) if
        manages(user, employee)
        and submitted(employee, expense);

    allow(user, "view", expense) if
        role(user, "manager", Project.by_id(expense.project_id);

.. note::
    For full examples of the patterns used here, check out the following guides:

    - :ref:`abac-basics`
    - :ref:`abac-hierarchies`
    - :ref:`abac-rbac`

.. todo::
    Keep going! Come up with the conclusion here for why the policy is great.

- This is declarative - why is that better?
- We don't need to worry about the search algorithm - why is this better?
    - you don't need to tell it how to combine things together, it's searching through everything you've told it, combining them to deduce / make that decision. you give it all the ingredients and it puts them together in the right order to make the decision
- Wait.. searching sounds expensive.
    - Link to performance discussion.
- Inferences - determining new properties over your data, like roles and relationships.
    - combine statements together to create new statements
- why is this better than doing it in code?


Authorization decisions and application data are inseparable
------------------------------------------------------------

.. todo::
    Fill out text on why authZ decisions are inseparable

Some things to cover:

- But your first principle was separation of concerns! Now you're saying they
  are inseparable?
- What works outside of an application? Roles. Things _only_ concerned with the user.
- Take our simplest rule: users can view expenses they submitted. Immediately requires
  both the user object and the expense. Sure, you can says users can view expenses, and handle this check in the app. But all of our above rules need access to that data.
- Something, something... By integrating so deeply with the application domain, we are able to stand on the shoulders of the existing model relationships, weaving together discrete bits of business logic into a rich authorization tapestry.

### Other TBD

- Would be good to include some kind of architecture graphic? What would it show?
- I think we should write somewhere on this page what the key pieces/artifacts of oso are - i.e., language, authz libraries, repl...just seems like a nuts n bolts kind of 'what is oso' question to answer, probably fairly early on
- Alex idea: go one step further than key pieces/artifacts: show what's under the hood, i.e., FFI...
- What is the right thing to link to as next step? I guess 'add oso to your app'?
