---
title: Where to Enforce
weight: 3
description: |
    Learn how to apply authorization at different layers of your application.
---

# Where to Enforce

## Where to apply authorization?

There are a few different where we can apply authorization controls. Applying authorization as early
as possible on the request path can help make sure that every action is authorized which can be a security win. On the other hand, if the decision needs to access application data and context, then it may not be possible.

In this case, keeping the authorization decision as close as possible to the _data_ can ensure that all access to data is done in a secure way, while additionally making available rich context for decisions.


In this guide, we will cover:

1. How to use Oso to protect data access over three different dimensions:
    * model (table)
    * record (row)
    * field (column)
2. Where to integrate Oso’s policy evaluation in your application.

## Access Control Dimensions

Authorization is fundamentally about controlling data access or modification
based on properties of the actor. This is why `actor` and `resource` are
primary concepts in Oso.

Control over data access is commonly exercised over several dimensions:


* **model** or table level (an actor can access expense data, but not project data)
* **record** or row level (an actor can access a particular expense, or an expense
matching certain conditions)
* **field** or column level (an actor can access or update only certain fields of a
record)

An Oso policy can restrict access along one or several of these dimensions.

```polar
allow(actor, "approve", expense: Expense) if
    actor = expense.submitted_by.manager
    and expense.amount < 10000;
```

The above policy controls access to the Expense model on the **record level**.
An actor can only approve an expense if they are the manager of the submitter
and the expense amount is below a certain limit.

### Primary and Secondary Authorization Data

This policy controls access to an `Expense`, the **primary authorization
data**.  It relies on other data to make the decision: the submitter of the
expense (`expense.submitted_by`), and the manager of the submitter
(`submitted_by.manager`).  This *other* data is called **secondary
authorization data**.  An important class of secondary authorization data is
**actor data**.  This data includes properties of the actor, like their role, or
team membership that is often used in controlling access regardless of whether
it is over rows, columns or fields.

### Where the Policy is Evaluated

Where the policy is evaluated has a significant impact on the granularity of
access control that is possible.  In the above example, we rely on the
`amount` field of the expense. Therefore, the `Expense` (the **primary
authorization data**) must be fetched from the application’s store when the rule
is evaluated.

```python
def get_expense(user, expense_id):
    expense = db.fetch(
        "SELECT * FROM expenses WHERE id = %", expense_id)

    if oso.is_allowed(user, "view", expense):
        # Process request
        ...
    else:
        # Not authorized
        return NotAuthorizedResponse()
```

This **policy evaluation point** is **after primary data fetch**. An
authorization decision is made after the primary data is fetched from the
persistence layer (be it a SQL database, an ORM, or another service) and can be
used to make an authorization decisions.

Alternatively, we could have placed our policy evaluation point **before primary
data fetch**. This would limit the power of our policy, since we would not be
able to check the `amount` field of the `Expense`. Keep reading to see how
we would apply this technique to **model level** and **field level** access
control.

## Model Level Access Control

Sometimes, access control does not rely on properties of the primary data.  This
type of access control is called **model level**.

```polar
allow(actor, "view", "expense") if actor.role = "accountant";
allow(actor, "modify", "team") if actor.role = "hr_admin";
allow(actor, "modify", "project") if actor.role = "hr_admin";
allow(actor, "modify", "organization") if actor.role = "hr_admin";
```

This brief policy shows an example of model level access control:


* An accountant can view expenses.
* HR admins can modify teams, projects, and organizations.

Notice that this policy does not rely on any **primary authorization data**.
Therefore it can be evaluated either before or after the primary data fetch.
Here’s what it would look like before:

```python
def get_expense(user, id):
    # See if the user is allowed to access expenses at all.
    if oso.is_allowed(user, "view", "expense"):
        expense = db.fetch(
            "SELECT * FROM expenses WHERE id = %", expense_id)
        # Process request
    else:
        # Not authorized
        return NotAuthorizedResponse()
```

**NOTE**: You may have noticed that this policy still accesses **actor data**.  This
is fine, since usually this data will be fetched prior to authorization as
part of the authentication flow.

## Record Level Access Control, Revisited

Our first example was an example of record level
access control. In general, record level access control must be performed
**after primary data fetch**. This holds true for actions that fetch, edit, or
delete primary data. (Our example above used the `"approve"` action, which
would result in an edit). An exception to this rule is actions that create
new data.

### Create Requests

```python
def create_expense(user, expense_data):
    # Create a new expense from the request.
    expense = Expense.from_json(expense_data)

    if oso.is_allowed(user, "create", expense):
        db.insert(expense)
        # Process rest of expense
    else:
        # Not authorized.
        return NotAuthorizedResponse()
```

Here, we were able to evaluate the policy **before data fetch** (modification in
this case). The data is already available from the user, before it is written to
the database.  Alternatively, if we are using a transactional data store, we
could evaluate the policy after inserting the data, as long as a rollback is
performed if the authorization fails:

```python
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
```

This may be helpful to keep code consistent across route handlers, or if the
database makes some transformation during insertion that impacts the
authorization logic.

## Field Level Access Control

In contrast to record level access control, field level access control
determines what portions of a given record can be accessed.

```polar
allow_field(actor, "view", _: Expense, "submitted_by");
allow_field(actor, "view", expense: Expense, "amount") if
    actor = expense.submitted_by;
allow_field(actor, "view", _: Expense, "amount") if
    actor.role = "accountant";
```

This policy uses a new rule, called `allow_field` to:

* Allow everyone to view the `submitted_by` field.
* Allow the submitter of the expense to view the `amount`.
* Allow actors with the `"accountant"` role to view the `amount` of any
expense.

We can combine field access control with our record level access control
example:

```python
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
```

**NOTE**: We use the `oso.query` method in this example to query a rule other than
`allow`.

In this example, we evaluated both record & column level access control after
data fetch.  However, it may be more efficient to use column level access
control to only load the columns the user can access:

```python
from oso import Variable

def get_expense(user, expense_id):
    # Query Oso for all fields allowed for this user.
    allowed_fields = oso.query_rule("allow_field",
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
```

Now, we are using Oso to tell us what fields to query for.  In this example, the
policy is evaluated both **before and after data fetch** for greater efficiency.

## Authorizing List Endpoints

A list endpoint can be challenging to authorize since it deals with obtaining
a collection of resources.  Often, the filter used to obtain these resources will
be related to the authorization policy.  For example, suppose we have the following
access control rule in our policy:

```polar
# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) if
    role(actor, "accountant") and
    actor.location = resource.location;
```

To authorize this request for a single record fetch, for example
`GET /expense/1`, we could fetch the record (the equivalent of
`SELECT \* FROM expenses WHERE id = 1`) then evaluate the allow rule, passing
the record to Oso as a resource.

A list endpoint involves multiple records that must be fetched from the data
layer, then authorized. Usually a filter must be applied when querying for
multiple records for performance reasons. We have a few options to perform
authorization:


1. Apply a less restrictive filter in application code (or no filter) and
individually authorize every record.
2. Duplicate our filtering in both places (application and policy).
3. Authorize the filter to be applied to the query before data fetch,
instead of the resource.
4. Have Oso output the filter to be applied to the query before data fetch.

Let’s see an example of how each of these would work. We will use Python
pseudocode for this example, but the same concepts translate to any web application.

### Authorizing each record individually

In this example, we apply a filter in our application (how restrictive this is
depends on the use case & expected amount of records).  For example, suppose each
user has an associated organization id.  Users can only view expenses by
organization.  We could apply this filter, then further restrict access using Oso.

```python
def get_expenses(user):
    records = db.fetch(
        "SELECT * FROM expenses WHERE organization_id = %s AND is_active = 't'",
                       user.organization_id)

    authorized_records = []

    # Use oso.is_allowed to filter records that are not authorized.
    for record in records:
        if not oso.is_allowed(actor=user, action="view", resource=record):
            continue

        authorized_records.append(record)
```

This approach works well if the expected size of `records` after the database
fetch is relatively small.  It allows the same policy to be used for GET & list
fetch requests.  It is not performant if the record set is large.

### Duplicating filter logic

Above, we only use Oso to confirm that access is allowed.  While Oso
remains the authoritative source of authorization information, it is not used
to determine which records to fetch.  This approach is helpful if you have
authorization rules that must be applied to highly sensitive data using Oso,
but still need the performance gains from explicitly filtering records
in your application.

```python
def get_expenses(user):
    # Check that user is authorized to list responses.
    if not oso.is_allowed(actor=user, "list", resource=Expense):
       return NotAuthorizedResponse()

    # Apply location filter for authorization, as well as other
    # non-authorization filters (is_active = 't')
    records = db.fetch(
        "SELECT * FROM expenses WHERE location_id = %s AND is_active = 't'",
        user.location_id)

    # Use oso.is_allowed to *confirm* that records are authorized.
    for record in records:
        if not oso.is_allowed(actor=user, action="view", resource=record):
            if DEBUG:
                # In debug mode, this is a programming error.
                # The logic in Oso should be kept in sync with the filters
                # in the above query.
                assert False

            raise NotAuthorizedResponse()
```

For the above example, we add the following to our policy:

```polar
# Accountants can list expenses
allow(actor: User, "list", resource: Expense) if
    role(actor, "accountant");
```

This takes the role check portion from the `view` rule and allows us to apply
it separately, before we authorize the query. This means we don’t need to fetch
expenses when the request would ultimately be denied because the role is not
allowed to list expenses.  The second `oso.is_allowed()` call confirms that the
filter applied in the database fetch produces records that are allowed by the
access policy.  With this approach, the policy duplicates database fetch logic
and must be manually kept in sync by developer.  To aid with this, we add an
assertion in debug mode.

### Authorizing the filter to be applied, instead of the resource

Instead of duplicating logic in Oso and our application, we could authorize the
request filter.

```python
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

    # Use ``query_rule`` to evaluate a rule that authorizes the filter.
    if not oso.query_rule("allow_filter", user, "view", "expense", auth_filters):
        return NotAuthorizedResponse()

    # This function converts our structured filter into a SQL WHERE statement
    # for execution.  If we are using an ORM this would be performed by the ORM.
    where, params = filters_to_sql(auth_filters)

    records = db.fetch(f"SELECT * FROM expenses WHERE {where} AND is_active = 't'",
                       params)

    # No additional authorization of records is needed since we checked the query.
```

To support this structure, our policy would look something like:

```polar
# Accountants can list expenses
allow(actor: User, "list", resource: Expense) if
    role(actor, "accountant");

# A set of filters is allowed for a view request as long as it
# restricts the location id properly.
allow_filter(actor, "view", "expense", filters) if
    ["location_id", "=", actor.location_id] in filters;
```

While we have abstracted the policy slightly further and no longer need
as many `oso.is_allowed()` checks to complete the request, we still must keep
the filter in sync between Oso and our code. Instead, we can make Oso the
authoritative source of query filters that perform authorization.

### Have Oso output the filter

This is a similar structure to above, but instead the authorization filter is
contained only in the policy.  This structure can simplify application code, and
allows for filters that are conditional on other attributes. For example, our
policy for “view” could contain the additional rule

```polar
# Users can view expenses they submitted
allow(actor: User, "view", resource: Expense) if
    resource.submitted_by = actor.name;

# Accountants can view expenses from their location
allow(actor: User, "view", resource: Expense) if
    role(actor, "accountant") and
    actor.location = resource.location;
```

We could instead refactor these rules so that they operate on filters:

```polar
allow_with_filter(actor: User, "view", "expense", filters) if
    filters = ["submitted_by", "=", actor.name];

allow_with_filter(actor: User, "view", "expense", filters) if
    role(actor, "accountant") and
    filters = ["location", "=", actor.location];
```

Now, in our app:

```python
def get_expenses(user):
    # Get authorization filters from Oso
    filters = oso.query_rule(
        "allow_with_filter", actor, "view", "expense", Variable("filters"))

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
```

This approach results in simpler authorization code, and the policy is truly
in full control of authorization.  It can be modified independently from
application code, without any duplication.

## Summary

In this guide, we covered the various access control levels
(model, attribute & field) and showed you how to integrate Oso in your application
at various spots. We then covered list endpoints — which are often difficult to
write complex authorization for — in detail. We discussed several potential
techniques for structuring a policy that handles these types of requests.
