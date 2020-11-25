=======================
List Filtering
=======================

The oso policy engine can now produce such filters from your policy. Below
we'll briefly explain how it works and link to instructions and examples for
the supported ORMs (currently Django & SQLAlchemy).

.. toctree::

    django
    sqlalchemy

How it works
============

Let's take our authorization rule from above: a user may view a post if it was
created by a friend:

.. code-block:: polar

  allow(user, "view", post) if post.creator in user.friends;

For a particular user, we can ask two fundamental questions:

1. Is that user allowed to view a specific post, say, ``Post{id: 1}``?
2. Which posts is that user allowed to view?

The answer to the first question is a boolean. The answer to the second is a
set of *constraints* that must hold in order for *any* ``Post`` to be
authorized.

oso can produce such constraints through *partial evaluation* of a policy.
Instead of querying with concrete object (e.g., ``Post{id: 1}``), you can pass
a ``Partial`` value, which signals to the engine that constraints should be
collected for it. A successful query for a ``Partial`` value returns constraint
expressions:

.. code-block:: polar

  constraints here

Partial evaluation is a generic capability of the oso engine, but making use of
it requires an adapter that translates the emitted constraint expressions into
database filters. Our first two supported adapters are for the :doc:`Django
</using/frameworks/django>` and :doc:`SQLAlchemy
</using/frameworks/sqlalchemy>` ORMs, with more on the way.

These adapters allow oso to effectively translate policy logic into SQL `WHERE`
clauses:

.. code-block:: sql

  SQL here

In effect, authorization is being enforced by the policy engine and the ORM
cooperatively.

.. image:: /getting-started/list-filtering/list-filtering.svg

Alternative solutions
=====================

Partial evaluation is not the only way to efficiently apply authorization to
collections of data. On the :doc:`Access
Patterns <getting-started/application/patterns>`
page, we describe [several
alternatives](https://docs.osohq.com/getting-started/application/patterns.html#authorizing-list-endpoints).
Manually applying `WHERE` clauses to reduce the search space (or using
[ActiveRecord-style
scopes](https://guides.rubyonrails.org/active_record_querying.html#scopes))
requires additional application code and still needs to iterate over a
potentially large collection. Authorizing the filter to be applied (or having
oso output the filter) doesn't require iterating over individual records, but
it does force you to write policy over filters instead of over application
types, which can lead to more complex policies and is a bit of a leaky
abstraction.

## Frameworks

- To learn more about this feature and see usage examples, see our ORM specific documentation:
    - Django
    - SQLAlchemy
    - Odoo (coming soon)

- More framework integrations are coming soon - join us in Slack to discuss your use case or open an issue on github.
