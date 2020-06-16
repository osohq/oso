==========================
Frequently Asked Questions
==========================


.. TODO: once we have some performance numbers
    Performance of oso
    ------------------

    The performance of oso depends almost entirely on two things:
    the structure of your Polar policy, and the time to lookup application data.

    At the time of writing, for some typical Polar workloads, the time
    to evaluate a query takes TODO: fill me in.

    For looking up application data, oso adds about 2us of overhead, per datum returned.
    In most cases, the lookup itself will be the most costly part.

The "N+1 Problem"
----------------

A core part of understanding how oso will perform under regular
workloads is recognising that oso applies a search algorithm to
evaluate the policy.

Since it is common in policies to iterate over members or attributes
in order to look for matching information, it can be common to encounter
variants of the 
`N+1 problem <https://medium.com/@bretdoucette/n-1-queries-and-how-to-avoid-them-a12f02345be5>`_.

For example, given the following Polar policy:

.. code-block:: polar

  has_grandchild_called(grandparent: Person, name) :=
      child in grandparent.children,
      grandchild in child.children,
      grandchild.name = name;

This can potentially exhibit this N+1 behaviour. It will first call
the `Person.children` method on the input grandparent, expecting a
list to iterate over with the `in` operator. This might translate
into a DB query by the application.

For each `child` returned from this method, `Person.children` is again
called, which may make another DB query, ultimately resulting in N+1
queries - one for the initial query, and one for each grandhchild.

The answer to solving this ultimately lies in how your application accesses
data. Since this problem is not unique to oso and authorization queries,
there already exist a few patterns for this, such as `eager-loading ORMs <https://guides.rubyonrails.org/active_record_querying.html#eager-loading-associations>`_
and `dataloaders <https://github.com/graphql/dataloader>`_ for `GraphQL <https://github.com/Shopify/graphql-batch>`_.

Here we will show how these patterns can be leveraged in oso.

**Option 1.**  Implement a lookup method that accepts as input a list of keys.

For example:

.. code-block:: python

    class Person:
        @classmethod
        def lookup_children(cls, ids: List[int]]):
            # select * from people where id in ids
            return children

.. code-block:: polar

    has_grandchild_called(grandparent: Person, name) :=
        children = grandparent.children, # gets the _list_ of children
        grandchild in Person.lookup_children(children.id),
        grandchild.name = name;

This has the benefit of being the simplest, and most explicit. But does not
leverage any data access abstractions. Any optimisation method works fine here,
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

    has_grandchild_called(grandparent: Person, name) :=
        child in grandparent.children.prefetch_related("children"),
        grandchild in child.children.all(),
        grandchild.name = name;

Since oso is able to work directly with native objects, using the
existing Django methods to prefetch the grandchildren in this case
can be applied directly where it's used.

.. TODO
    3. *Coming soon*: Polar SQL query builder

    One way to avoid this is to directly connect Polar to your SQL database
    and allow it to optimise the queries.

    See: https://www.cs.cmu.edu/afs/cs/project/ai-repository/ai/lang/prolog/code/io/pl2sql/0.html


.. TODO: profiling tool
    Detecting performance issues
    ----------------------------

    In order to facilitate understanding and debugging performance
    issues like the above, oso includes simple profiling functionality.
    On making a query, add the `profile=True` paramter. When a trace is
    returned for a query, you can see where the majority of time was spent.

    This information can be viewed with the oso trace viewer.
