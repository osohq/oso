
The "N+1 Problem"
-----------------

A core part of understanding how oso will perform under regular
workloads is recognizing that oso applies a search algorithm to
evaluate the policy.

Since it is common in policies to iterate over members or attributes
in order to look for matching information, it can be common to encounter
variants of the
`N+1 problem <https://medium.com/@bretdoucette/n-1-queries-and-how-to-avoid-them-a12f02345be5>`_.

For example, given the following policy:

.. code-block:: polar
    :caption: :fa:`oso` children.polar

    has_grandchild_called(grandparent: Person, name) if
        child in grandparent.children and
        grandchild in child.children and
        grandchild.name = name;

This can potentially exhibit this N+1 behavior. It will first call
the `Person.children` method on the input grandparent, expecting a
list to iterate over with the `in` operator. This might translate
into a DB query by the application.

For each `child` returned from this method, `Person.children` is again
called, which may make another DB query, ultimately resulting in N+1
queries - one for the initial query, and one for each grandchild.

The answer to solving this ultimately lies in how your application accesses
data. Since this problem is not unique to oso and authorization queries,
there already exist a few patterns for this, such as `eager-loading ORMs <https://guides.rubyonrails.org/active_record_querying.html#eager-loading-associations>`_
and `dataloaders <https://github.com/graphql/dataloader>`_ for `GraphQL <https://github.com/Shopify/graphql-batch>`_.

Here we will show how these patterns can be leveraged in oso.

**Option 1.**  Implement a lookup method that accepts as input a list.

For example:

.. code-block:: python
    :caption: :fab:`python` person.py

    class Person:
        @classmethod
        def batch_lookup_children(cls, people: List[Person]):
            parent_ids = [p.id for p in people]
            children = db.query(
                "select id, name from people, children \
                    where people.id = children.child_id, children.parent_id in ?", 
                parent_ids
            )
            return children

.. code-block:: polar
    :caption: :fa:`oso` children.polar

    has_grandchild_called(grandparent: Person, name) if
        children = grandparent.children # gets the _list_ of children
        and grandchild in Person.batch_lookup_children(children)
        and grandchild.name = name;

This has the benefit of being the simplest, and most explicit. But does not
leverage any data access abstractions. Any optimization method works fine here,
for example if this were a sufficiently common use case, then a `grandchildren`
method and DB index could be added to improve performance.

**Option 2.** Implement some form of dataloader/eager loading in your application.

This is the common approach to solve these in ORMs, like Ruby on Rails.
The `Person` model in this case could be configured to eager load children
when looking up records. In this case, each `child` record returned
by the `grandparent.children` method call will already have loaded and
locally stored the `child.children` attribute.

For example, in a Django application you might write:

.. code-block:: polar
    :caption: :fa:`oso` children.polar

    has_grandchild_called(grandparent: Person, name) if
        child in grandparent.children.prefetch_related("children")
        and grandchild in child.children.all()
        and grandchild.name = name;

Since oso is able to work directly with native objects, using the
existing Django methods to prefetch the grandchildren in this case
can be applied directly where it's used.
