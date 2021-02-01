.. meta::
   :description: Learn about how to control access at different levels of access -- at the model, record and field level.

====================
Authorization Levels
====================

.. role:: sql(code)
   :language: psql
   :class: highlight

.. highlight:: polar

Oso supports doing access control at different levels.
In this guide, we will cover:

1. How to use oso to protect data access over three different dimensions:

   - model (table)
   - record (row)
   - field (column)

2. Where to integrate oso's policy evaluation in your application.


Access Control Dimensions
=========================

Authorization is about who can do what on your data. In Oso we call the "who"
the ``actor``, what they can do the ``action`` and the data they're doing the
action on the ``resource``. (For more on Oso authorization terminology see :doc:`primary concepts </more/glossary>`)

Control over data is commonly done at different dimensions:

- **model** or table level (an actor can access expense data, but not project data)
- **record** or row level (an actor can access a particular expense, or an expense
  matching certain conditions)
- **field** or column level (an actor can only update only the expense status, not
  it's amount)

An oso policy can restrict access along one or several of these dimensions.

.. _first-record-level:

.. code-block:: polar
  :caption: :fa:`oso`

  allow(actor, "approve", expense: Expense) if
      actor = expense.submitted_by.manager
      and expense.amount < 10000;

The above policy controls access to the Expense model on the **record level**.
An actor can only approve an expense if they are the manager of the submitter
and the expense amount is below a certain limit.

Model Level Access Control
==========================

Sometimes, access control does not depend on the properties of the resource but just
on it's type. This is called **model level** can be used for coarse level authorization.
This is useful if you want to restrict access to a whole page if the user cannot see any
of the resources of the kind the page is about. Since the rules depend only on the type
of the model and not on the data it makes it easier to add these checks upstream in
middleware.

.. code-block:: polar
    :caption: :fa:`oso`

    allow(actor, "view", "Expense") if actor.role = "accountant";
    allow(actor, "modify", "Team") if actor.role = "hr_admin";
    allow(actor, "modify", "Project") if actor.role = "hr_admin";
    allow(actor, "modify", "Organization") if actor.role = "hr_admin";

This brief policy shows an example of model level access control:

- An accountant can view expenses.
- HR admins can modify teams, projects, and organizations.

In this example we are representing the model type as a string and writing the rule
over that.

.. code-block:: python
    :caption: :fab:`python`

    def expense_page(user, id):
        # See if the user is allowed to access expenses at all.
        if not oso.is_allowed(user, "view", "Expense"):
            return NotAuthorizedResponse()
        # Process request

Model level is also sometimes a good way to write rules about creating new resources.
You can write a rule saying if a user can create an Expense or not.

Record Level Access Control
===========================

Our :ref:`first example <first-record-level>` was an example of record level
access control. Record level is about authorizing an action on a specific instance
of a resource. Is the user allowed to edit **this** Expense. Record level
rules are passed the instance and can check properties on it.

Record level checks can also be used for create. When you're creating a new record you
can create the instance, then check if it's an instance you are allowed to create and then
save it to the database if it is.

Create Requests
---------------

.. _second-record-level:

.. code-block:: python
    :caption: :fab:`python`

    def create_expense(user, expense_data):
        # Create a new expense from the request.
        expense = Expense.from_json(expense_data)

        if oso.is_allowed(user, "create", expense):
            db.insert(expense)
            # Process rest of expense
        else:
            # Not authorized.
            return NotAuthorizedResponse()


Field Level Access Control
==========================

In contrast to record level access control, field level access control
determines what portions of a given record can be accessed.

.. code-block:: polar
    :caption: :fa:`oso`

    allow_field(actor, "view", _: Expense, "submitted_by");
    allow_field(actor, "view", expense: Expense, "amount") if
        actor = expense.submitted_by;
    allow_field(actor, "view", _: Expense, "amount") if
        actor.role = "accountant";

This policy uses a new :ref:`rule <polar-rules>`, called ``allow_field`` to:

- Allow everyone to view the ``submitted_by`` field.
- Allow the submitter of the expense to view the ``amount``.
- Allow actors with the ``"accountant"`` role to view the ``amount`` of any
  expense.

We can combine field access control with our record level access control
:ref:`example <second-record-level>`:

.. code-block:: python
    :caption: :fab:`python`

    def get_expense(user, expense_id):
        expense = db.fetch(
            "SELECT * FROM expenses WHERE id = %", expense_id)

        # Record level authorization.
        if oso.is_allowed(user, "view", expense):
            authorized_data = {}

            for field, value in expense.items():
                # Check if each field in the expense is allowed, and only
                # include those that are in authorized_data.
                if oso.query_rule("allow_field", actor, "view", expense, field):
                    authorized_data[field] = value

            # Return only authorized_data to the user.
            ...
        else:
            # Not authorized
            return NotAuthorizedResponse()

.. note::

    We use the ``oso.query`` method in this example to query a rule other than
    ``allow``.

We could extend the field_allow rule to take in an additional value. This would let us
write rules for updates over what the field will be updated to as well as it's current
value.

.. code-block:: polar
    :caption: :fa:`oso`

    allow_field(actor, "update", _: Expense, "assigned_to", new_assigned_to) if
        new_assigned_to = actor.id;

This would allow a user to update the assigned_to field on an Expense but only if they
are updating it to themselves. By adding this additional value to ``allow_field`` we can
now write logic that checks all the information we have about the update.

Summary
=======

In this guide, we covered the various access control levels
(model, record & field) and showed you how to integrate oso in your application
at various spots.

.. admonition:: What's next
    :class: tip whats-next

    * Explore :doc:`/using/examples/index` in depth.
    * Read more about writing oso policies:
      :doc:`/getting-started/policies/index`.
