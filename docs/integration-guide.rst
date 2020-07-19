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

Use an allow rule with no resource conditions to control access on a model level::

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

Then, the application would make several ``oso.allow()`` calls before permitting
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

- **before primary data fetch**: An authorization decision is made before
  primary data is fetched from the persistence layer. Primary data is not
  available as context during the authorization decision.
- **after primary data fetch**: An authorization data is made after the primary
  data is fetched from the persistence layer and can be used to make an
  authorization decisions.

Table / model level
-------------------

This type of authorization is easy to do before data fetch, all that is required
to make a decision is the model name to authorize.  It may also be performed
after data fetch by checking the class name or a resource field that indicates
the type of the data.

Row / attribute level
---------------------

Row & attribute level access control by necessity requires access to the data
being authorized. For most types of requests, this authorization must be
performed after a primary data fetch. Authorizing a GET request for a single record
requires that record's data before the authorization can be evaluated. An update
or delete request requires the same data.  A create request is the exception
to this rule, since it can be authorized on the basis of the data to be created
before committing it to the data store.

Column / field level
--------------------

Authorizing access to columns can be done before or after data access. If
performed before, the columns to be accessed in a read query or updated can be
authorized.  If performed after, the data could be masked based on the columns
that are allowed to be read.

Authorizing list endpoints
--------------------------

A list endpoint can be challenging to authorize since it deals with obtaining
a collection of resources.  Often the filter used to obtain these resources will
be related to the authorization policy.  For example, suppose we have the following
access control rule in our policy::

    # Accountants can view expenses from their location
    allow(actor: User, "view", resource: Expense) if
        role(actor, "accountant") and
        actor.location = resource.location;

To authorize this request for a single record fetch, for example
``GET /expense/1``, we could fetch the record (the equivalent of
``SELECT * FROM expenses WHERE id = 1``) then evaluate the allow rule, passing
the record to oso as a resource.

A list endpoint involves multiple records that must be fetched from the data
layer, then authorized. Usually a filter must be applied when querying for
multiple records for performance reasons. We have a few options to perform
authorization:

    1. Apply a less restrictive filter in application code (or no filter) and
       individually authorize every record.
    2. Duplicate our filtering in both places (application and Polar).
    3. Authorize the filter to be applied to the query before data fetch,
       instead of the resource.
    4. Have oso output the filter to be applied to the query before data fetch.

Let's see an example of how each of these would work. We will use Python
pseudocode for this example, but the same concepts translate to any web application.

**Authorizing each record individually**

In this example, we apply a filter in our application (how restrictive this is
depends on the use case & expected amount of records).  For example, suppose each
user has an associated organization id.  Users can only view expenses by
organization.  We could apply this filter, then further restrict access using oso.


.. code-block:: python

    def get_expenses(user):
        records = db.fetch(
            "SELECT * FROM expenses WHERE organization_id = %s AND is_active = 't'",
                           user.organization_id)

        authorized_records = []

        # Use oso.allow to filter records that are not authorized.
        for record in records:
            if not oso.allow(actor=user, action="view", resource=record):
                continue

            authorized_records.append(record)

This approach works well if the expected size of ``records`` after the database
fetch is relatively small.  It allows the same policy to be used for GET & list
fetch requests.  It is not performant if the record set is large.

**Duplicating filter logic**

Above, we only use oso to confirm that access is allowed.  While oso
remains the authoritative source of authorization information, it is not used
to determine which records to fetch.  This approach is helpful if you have
authorization rules that must be applied to highly sensitive data using oso,
but still need the performance gains from explicitly filtering records
in your application.

.. todo::
    Below example doesn't actually work because a class does not match a
    rule (only an instance will).

.. code-block:: python

    def get_expenses(user):
        # Check that user is authorized to list responses.
        if not oso.allow(actor=user, "list", resource=Expense):
           return NotAuthorizedResponse()

        # Apply location filter for authorization, as well as other
        # non-authorization filters (is_active = 't')
        records = db.fetch(
            "SELECT * FROM expenses WHERE location_id = %s AND is_active = 't'",
            user.location_id)

        # Use oso.allow to *confirm* that records are authorized.
        for record in records:
            if not oso.allow(actor=user, action="view", resource=record):
                if DEBUG:
                    # In debug mode, this is a programming error.
                    # The logic in oso should be kept in sync with the filters
                    # in the above query.
                    assert False

                raise NotAuthorizedResponse()

For the above example, we add the following to our policy::

    # Accountants can list expenses
    allow(actor: User, "list", resource: Expense) if
        role(actor, "accountant");

This takes the role check portion from the ``view`` rule and allows us to apply
it separately, before we authorize the query. This means we don't need to fetch
expenses when the request would ultimately be denied because the role is not
allowed to list expenses.  The second ``oso.allow()`` call confirms that the
filter applied in the database fetch produces records that are allowed by the
access policy.  With this approach, the policy and database fetch logic is
duplicative and must be manually kept in sync by developer.  To aid with this,
we add an assertion in debug mode.

**Authorizing the filter to be applied, instead of the resource**

Instead of duplicating logic in oso and our application, we could authorize the
request filter.

.. code-block:: python

    def get_expenses(user):
        # Check that user is authorized to list responses.
        if not oso.allow(actor=user, "list", resource=Expense):
           return NotAuthorizedResponse()

        # Structured format representing WHERE clauses.
        # In an ORM, we might use the ORM's native query construction objects
        # to represent this.
        auth_filters = [
            ("location_id", "=", user.location_id)
        ]

        # Use ``query_pred`` to evaluate a rule that authorizes the filter.
        if not oso.query_pred("allow_filter", user, "view", Expense, auth_filters):
            return NotAuthorizedResponse()

        # This function converts our structured filter into a SQL WHERE statement
        # for execution.  If we are using an ORM this would be performed by the ORM.
        where, params = filters_to_sql(auth_filters)

        records = db.fetch(f"SELECT * FROM expenses WHERE {where} AND is_active = 't'",
                           params)

        # No additional authorization of records is needed since we checked the query.

.. todo::
    We have no way to expect an Expense class as a specializer. We may need
    some syntax for that.

.. todo::
    It would be nice if the filter structure can actually be evaluated
    by Polar for "view" queries, but that would require some complicated
    metaprogramming type stuff, or at least a getattr style predicate.

To support this structure, our Polar policy would look something like::

    # Accountants can list expenses
    allow(actor: User, "list", resource: Expense) if
        role(actor, "accountant");

    # A set of filters is allowed for a view request as long as it
    # restricts the location id properly.
    allow_filter(actor, "view", resource_type: Expense, filters) if
        ["location_id", "=", actor.location_id] in filters;

While we have abstracted the policy slightly further and no longer need
as many ``oso.allow()`` checks to complete the request, so must keep
the filter in sync between oso and our code. Instead, we can make oso the
authoritative source query filters that perform authorization.

**Have oso output the filter**

This is a similar structure to above, but instead the authorization filter is
stored in Polar.  This structure can simplify application code, and allows for
filters that are conditional on other attributes. For example, our policy for
"view" could contain the additional rule

.. code-block:: polar
    :emphasize-lines: 1-3

    # Users can view expenses they submitted
    allow(actor: User, "view", resource: Expense) if
        resource.submitted_by = actor.name;

    # Accountants can view expenses from their location
    allow(actor: User, "view", resource: Expense) if
        role(actor, "accountant") and
        actor.location = resource.location;

We could instead refactor these rules so that they operate on filters::

    allow_with_filter(actor: User, "view", resource: Expense, filters) if
        filters = ["submitted_by", "=", actor.name];

    allow_with_filter(actor: User, "view", resource: Expense, filters) if
        role(actor, "accountant") and
        filters = ["location", "=", actor.location];

Now, in our app:

.. code-block:: python

    def get_expenses(user):
        # Get authorization filters from oso
        filters = oso.query_pred(
            "allow_with_filter", actor, "view", resource, Variable("filters"))

        # There may be multiple allow rules that matched, so we iterate over all
        # of them.  In the above example, every user can view expenses they submitted,
        # and accountants and view those in the same location as them.
        authorized_records = []
        for filter_set in filters.results:
            # This is the same conversion function from earlier.
            where, params = filters_to_sql(filter_set)
            records = db.fetch(
                f"SELECT * FROM expenses WHERE {where} AND is_active = 't'",
                params)

            authorized_records += records

        # No further authorization is necessary.

This approach results in simpler authorization code, and the policy is truly
in full control of authorization.  It can be modified independently from
application code, without any duplication.

Conclusion
----------

In this guide, we covered the various access control levels
(model, attribute & field) and showed you how to integrate oso in your application
at various spots. We then covered list endpoints -- which are often difficult to
write complex authorization for -- in detail. We discussed several potential
techniques for structuring a policy that handles these types of requests.

.. todo::
    what to read next
