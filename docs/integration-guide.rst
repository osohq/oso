===================
oso design patterns
===================

.. highlight:: polar

oso is extremely flexible, and supports numerous authorization control points.  In this guide, we
will cover:

1. How to use oso to protect data access over three different dimensions:
   - table or model
   - row or attribute based
   - column or field masking
2. Where to integrate oso's policy evaluation in your application.


Data access control levels
==========================

Authorization is fundamentally about controlling data access or modification
based on properties of the actor. This is why ``actor`` and ``resource`` are
primary concepts in oso's allow rules.

Control over data access is commonly exercised over several dimensions:

- table or model level (an actor can access expense data, but not project data)
- row or record level (an actor can access a particular expense, or an expense
  matching certain conditions)
- column or field level (an actor can access or update only certain fields of a
  record)

Polar permits the encoding of access control along each of these dimensions.

Model level
-----------

Use an allow rule with no resource conditions to control access on a model level

::

    allow(actor, "view", _: Expense);

This rule permits any actor to perform the view action on an ``Expense``. The
body could contain actor conditions & still be considered a model-level access
rule.

::

    allow(actor, "view", _: Expense) if
        actor.superuser = true;

Alternatively, a string could be used as the resource name::

    allow(actor, "view", "Expense");

Attribute level
---------------

Attribute level access control is a natural extension of model level access
control. Simply add additional conditions to the rule body that restrict access
on properties of the resource being accessed::

    allow(actor, "view", expense: Expense) if
        actor.superuser = true and expense.private = false;

Field level
-----------

Field level access control is along a different dimension than row or model
level access. There are several ways to represent this in Polar.

**String actions**

The name of the column to read could be encoded as a string action::

    # Row & model rules
    allow(actor, "view", _: Expense);

    # Column rules
    allow(actor, "amount", _: Expense);
    allow(actor, "submitted_by", _: Expense);

Then, the application would make several `oso.allow()` calls before permitting
access:

    - one for model or row access with the "view" action
    - one *for each* column being accessed using the "view" action.

**Compound actions**

This structure is simple, but does not allow us to encode more complex policies.
For example, suppose we allow a column to be read but not updated by a user.

We could use a compound data structure (either a dictionary or an application
class) to represent the action.  This would permit more fine-grained decisions::

    # Determine if columns in test are all in allowedColumns
    intersection(test, allowedColumns) if
        forall(column in test, test in allowedColumns);

    # A superuser can view any column.
    # We do not check columns, and bind it to an undefined variable using
    # pattern matching in the rule head.
    allow(actor, _: {action: "view", columns: _}, expense: Expense) if
        actor.superuser = true;

    # A regular user can only view the amount and location columns.
    allow(actor, _: {action: "view", columns: columns}, expense: Expense) if
        intersection(columns, ["amount", "location"]);

**Resource attributes**

Instead of encoding the columns in the action, we could encode them in the
resource. This can be helpful depending upon how oso is integrated with your
application::

    allow(actor, "view", expense: Expense{columns: columns}) if
        intersection(columns, ["amount", "location"];

This requires the resource class ``Expense`` to have an attribute or method that
returns all columns present in the expense.

Policy evaluation points
========================

Policy evaluation is performed by running a Polar query from within your
application.  This query can be integrated anywhere during the request
processing flow. We will discuss several possible points for each of the above
access control types.  Which you choose depends on the structure of your
application, and your authorization requirements.

There are several possible integration points for oso.  First some definitions:

- *primary authorization data*: The data being requested or modified during the
  course of the request.  Usually the request resource.
- *secondary authorization data*: Contextual data required to make the
  authorization decision that is not directly related to the particular request.
  This could be relational data describing the relationship between the actor &
  the resource, or information about the actor that is relevant to
  authorization, for example its team memberships.

Policy evaluation points:

- before primary data fetch
- after primary data fetch

Table / model level
-------------------

This type of authorization is easy to do before data fetch.  However, it may be
performed after data fetch by checking the class name or a resource field that
indicates the type of the data.


