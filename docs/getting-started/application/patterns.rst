===============
Access Patterns
===============

.. role:: sql(code)
   :language: psql
   :class: highlight

.. highlight:: polar

oso supports numerous access control schemes.
In this guide, we will cover:

1. How to use oso to protect data access over three different dimensions:

   - model (table)
   - record (row)
   - field (column)

2. Where to integrate oso's policy evaluation in your application.


Access control dimensions
=========================

Authorization is fundamentally about controlling data access or modification
based on properties of the actor. This is why ``actor`` and ``resource`` are
:doc:`primary concepts </more/key-concepts>` in oso.

Control over data access is commonly exercised over several dimensions:

- **model** or table level (an actor can access expense data, but not project data)
- **record** or row level (an actor can access a particular expense, or an expense
  matching certain conditions)
- **field** or column level (an actor can access or update only certain fields of a
  record)

An oso policy can restrict access along one or several of these dimensions.

.. _first-record-level:

.. code-block:: polar
  :caption: :fa:`oso`

  allow(actor, "approve", expense: Expense) if
      actor = expense.submitted_by.manager
      and expense.amount < 10000;

The below policy controls access to the Expense model on the **record level**.
An actor can only approve an expense if they are the manager of the submitter
and the expense amount is below a certain limit.

Primary and secondary authorization data
----------------------------------------

This policy controls access to an ``Expense``, the **primary authorization
data**.  It relies on other data to make the decision: the submitter of the
expense (``expense.submitted_by``), and the manager of the submitter
(``submitted_by.manager``).  This *other* data is called **secondary
authorization data**.  An important class of secondary authorization data is
**actor data**.  This data includes properties of the actor, like their role, or
team membership that is often used in controlling access regardless of whether
it is over rows, columns or fields.

Where the policy is evaluated
-----------------------------

Where the policy is evaluated has a significant impact on the level of access
control that is possible.  In the above example, we rely on the ``amount`` field
of the expense. Therefore, the ``Expense`` (the **primary authorization data**)
must be fetched from the application's store when the rule is evaluated.

.. _second-record-level:

.. code-block:: python
    :caption: :fab:`python`

    def get_expense(user, expense_id):
        expense = db.fetch(
            "SELECT * FROM expenses WHERE id = %", expense_id)

        if oso.is_allowed(user, "view", expense):
            # Process request
            ...
        else:
            # Not authorized
            return NotAuthorizedResponse()

This **policy evaluation point** is **after primary data fetch**. An
authorization decision is made after the primary data is fetched from the
persistence layer (be it a SQL database, an ORM, or another service) and can be
used to make an authorization decisions.

Alternatively, we could have placed our policy evaluation point **before primary
data fetch**. This would limit the power of our policy, since we would not be
able to check the ``amount`` field of the ``Expense``. Keep reading to see how
we would apply this technique to **model level** and **field level** access
control.

.. tip::

    A best practice for this type of access control is to integrate the policy
    evaluation point within the **data access layer** (the **M** in MVC).  This
    ensures that the ``oso.allow`` call is made when an expense is accessed, no
    matter where that access occurs in the application.

.. todo::

   Would be nice to be able to say here: "see this guide for an
   example of how to do this..."

Model level access control
==========================

Sometimes, access control does not rely on properties of the primary data.  This
type of access control is called **model level**.

.. code-block:: polar
    :caption: :fa:`oso`

    allow(actor, "view", "expense") if actor.role = "accountant";
    allow(actor, "modify", "team") if actor.role = "hr_admin";
    allow(actor, "modify", "project") if actor.role = "hr_admin";
    allow(actor, "modify", "organization") if actor.role = "hr_admin";

This brief policy shows an example of model level access control:

- An accountant can view expenses.
- HR admins can modify teams, projects, and organizations.

Notice that this policy does not rely on any **primary authorization data**.
Therefore it can be evaluated either before or after the primary data fetch.
Here's what it would look like before:

.. code-block:: python
    :caption: :fab:`python`

    def get_expense(user, id):
        # See if the user is allowed to access expenses at all.
        if oso.is_allowed(user, "view", "expense"):
            expense = db.fetch(
                "SELECT * FROM expenses WHERE id = %", expense_id)
            # Process request
        else:
            # Not authorized
            return NotAuthorizedResponse()

.. note::

    You may have noticed that this policy still accesses **actor data**.  This
    is fine, since usually this data will be fetched prior to authorization as
    part of the authentication flow.


Record level access control, revisited
======================================

Our :ref:`first example <first-record-level>` was an example of record level
access control. In general, record level access control must be performed
**after primary data fetch**. This holds true for actions that fetch, edit, or
delete primary data. (Our example above used the ``"approve"`` action, which
would result in an edit). An exception to this rule is actions that create
new data.

Create requests
---------------

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

Here, we were able to evaluate the policy **before data fetch** (modification in
this case). The data is already available from the user, before it is written to
the database.  Alternatively, if we are using a transactional data store, we
could evaluate the policy after inserting the data, as long as a rollback is
performed if the authorization fails:

.. todo::

    Would it be better to use a different term so I don't need the
    "(modification in this case)" phrase? Maybe before data access?

.. code-block:: python
    :caption: :fab:`python`

    def create_expense(user, expense_data):
        # Create a new expense from the request.
        expense = Expense.from_json(expense_data)

        inserted_record = db.insert(expense)
        if oso.is_allowed(user, "create", inserted_record):
            # Process rest of expense
        else:
            db.rollback()
            # Not authorized.
            return NotAuthorizedResponse()

.. todo:: Should this be paired with a policy?

This may be helpful to keep code consistent across route handlers, or if the
database makes some transformation during insertion that impacts the
authorization logic.

.. todo::

   Could write a section here on more complicated edit authorizations.
   Like a user is only allowed to change the project of an expense if they are a
   member of both the old and new project.

.. tip::

    This rollback technique can be applied to any request that modifies data and
    requires authorization. It may be particularly helpful for edit requests
    that edit and return the new version of data in the same data store query.
    (An :sql:`UPDATE ... RETURNING` query in SQL.)


Field level access control
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

This policy uses a new rule, called ``allow_field`` to:

- Allow everyone to view the ``submitted_by`` field.
- Allow the submitter of the expense to view the ``amount``.
- Allow actors with the ``"accountant"`` role to view the ``amount`` of any
  expense.

We can combine this access control with our record level access control
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
                if oso.query_predicate("allow_field", actor, "view", expense, field):
                    authorized_data[field] = value

            # Return only authorized_data to the user.
            ...
        else:
            # Not authorized
            return NotAuthorizedResponse()

.. note::

    We use the ``oso.query`` method in this example to query a rule other than
    ``allow``.

.. todo::

    relevant link & this is incorrect with our API now!

In this example, we evaluated both record & column level access control after
data fetch.  However, it may be more efficient to use column level access
control to only load the columns the user can access:

.. code-block:: python
    :caption: :fab:`python`

    from oso.api import Variable

    def get_expense(user, expense_id):
        # Query oso for all fields allowed for this user.
        allowed_fields = oso.query_predicate("allow_field",
                                   user, "view", expense, Variable("field"))
        # Convert the returned query response into a list of fields
        allowed_fields = [r["field"] for r in allowed_fields]
        allowed_fields_sql = db.sql_escape(allowed_fields.join(", "))

        expense = db.fetch(
            f"SELECT {allowed_fields_sql} FROM expenses WHERE id = %",
            expense_id)

        # Record level authorization.
        if oso.is_allowed(user, "view", expense):
            # Return only authorized_data to the user.
            ...
        else:
            # Not authorized
            return NotAuthorizedResponse()

Now, we are using oso to tell us what fields to query for.  In this example, the
policy is evaluated both **before and after data fetch** for greater efficiency.

.. admonition:: Variables provide flexibility

    Notice that we didn't have to change our policy file at all to make this
    change from the previous example. We used the ``Variable`` class which
    instructs oso to find all values of ``field`` that match the rules we defined
    in our policy.  This flexibility derives directly from writing a
    :doc:`declarative policy in Polar </more/language/polar-foundations>`!

Authorizing list endpoints
==========================

A list endpoint can be challenging to authorize since it deals with obtaining
a collection of resources.  Often the filter used to obtain these resources will
be related to the authorization policy.  For example, suppose we have the following
access control rule in our policy:

.. code-block:: polar
    :caption: :fa:`oso`

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
    2. Duplicate our filtering in both places (application and policy).
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
    :caption: :fab:`python`

    def get_expenses(user):
        records = db.fetch(
            "SELECT * FROM expenses WHERE organization_id = %s AND is_active = 't'",
                           user.organization_id)

        authorized_records = []

        # Use oso.allow to filter records that are not authorized.
        for record in records:
            if not oso.is_allowed(actor=user, action="view", resource=record):
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
    :caption: :fab:`python`

    def get_expenses(user):
        # Check that user is authorized to list responses.
        if not oso.is_allowed(actor=user, "list", resource=Expense):
           return NotAuthorizedResponse()

        # Apply location filter for authorization, as well as other
        # non-authorization filters (is_active = 't')
        records = db.fetch(
            "SELECT * FROM expenses WHERE location_id = %s AND is_active = 't'",
            user.location_id)

        # Use oso.allow to *confirm* that records are authorized.
        for record in records:
            if not oso.is_allowed(actor=user, action="view", resource=record):
                if DEBUG:
                    # In debug mode, this is a programming error.
                    # The logic in oso should be kept in sync with the filters
                    # in the above query.
                    assert False

                raise NotAuthorizedResponse()

For the above example, we add the following to our policy:

.. code-block:: polar
    :caption: :fa:`oso`

    # Accountants can list expenses
    allow(actor: User, "list", resource: Expense) if
        role(actor, "accountant");

This takes the role check portion from the ``view`` rule and allows us to apply
it separately, before we authorize the query. This means we don't need to fetch
expenses when the request would ultimately be denied because the role is not
allowed to list expenses.  The second ``oso.is_allowed()`` call confirms that the
filter applied in the database fetch produces records that are allowed by the
access policy.  With this approach, the policy and database fetch logic is
duplicative and must be manually kept in sync by developer.  To aid with this,
we add an assertion in debug mode.

**Authorizing the filter to be applied, instead of the resource**

Instead of duplicating logic in oso and our application, we could authorize the
request filter.

.. code-block:: python
    :caption: :fab:`python`

    def get_expenses(user):
        # Check that user is authorized to list responses.
        if not oso.is_allowed(actor=user, "list", resource=Expense):
           return NotAuthorizedResponse()

        # Structured format representing WHERE clauses.
        # In an ORM, we might use the ORM's native query construction objects
        # to represent this.
        auth_filters = [
            ("location_id", "=", user.location_id)
        ]

        # Use ``query_predicate`` to evaluate a rule that authorizes the filter.
        if not oso.query_predicate("allow_filter", user, "view", Expense, auth_filters):
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

To support this structure, our policy would look something like:

.. code-block:: polar
    :caption: :fa:`oso`

    # Accountants can list expenses
    allow(actor: User, "list", resource: Expense) if
        role(actor, "accountant");

    # A set of filters is allowed for a view request as long as it
    # restricts the location id properly.
    allow_filter(actor, "view", resource_type: Expense, filters) if
        ["location_id", "=", actor.location_id] in filters;

While we have abstracted the policy slightly further and no longer need
as many ``oso.is_allowed()`` checks to complete the request, so must keep
the filter in sync between oso and our code. Instead, we can make oso the
authoritative source query filters that perform authorization.

**Have oso output the filter**

This is a similar structure to above, but instead the authorization filter is
contained in the policy.  This structure can simplify application code, and
allows for filters that are conditional on other attributes. For example, our
policy for "view" could contain the additional rule

.. code-block:: polar
    :caption: :fa:`oso`
    :emphasize-lines: 1-3

    # Users can view expenses they submitted
    allow(actor: User, "view", resource: Expense) if
        resource.submitted_by = actor.name;

    # Accountants can view expenses from their location
    allow(actor: User, "view", resource: Expense) if
        role(actor, "accountant") and
        actor.location = resource.location;

We could instead refactor these rules so that they operate on filters:

.. code-block:: polar
    :caption: :fa:`oso`

    allow_with_filter(actor: User, "view", resource: Expense, filters) if
        filters = ["submitted_by", "=", actor.name];

    allow_with_filter(actor: User, "view", resource: Expense, filters) if
        role(actor, "accountant") and
        filters = ["location", "=", actor.location];

Now, in our app:

.. code-block:: python
    :caption: :fab:`python`

    def get_expenses(user):
        # Get authorization filters from oso
        filters = oso.query_predicate(
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
==========

In this guide, we covered the various access control levels
(model, attribute & field) and showed you how to integrate oso in your application
at various spots. We then covered list endpoints -- which are often difficult to
write complex authorization for -- in detail. We discussed several potential
techniques for structuring a policy that handles these types of requests.

.. todo::
    what to read next
