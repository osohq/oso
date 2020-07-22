.. _repl:

==============
The Polar REPL
==============

.. todo::
    Update the REPL guide

Developers can query Polar knowledge bases from the command line using the
Polar REPL (Read, Evaluate, Print, Loop). To run the REPL, first make sure
you have :doc:`installed oso </getting-started/download/index>`.

Once oso is installed, we can launch the REPL from the terminal::

    python3 -m polar.parser --interactive <policy files>

.. Both Python and Polar files can be loaded into the REPL.


Let's start by loading a simple Polar policy::

    # policy.polar
    allow(actor, "read", resource) if owns(actor, resource);
    owns("foo", "bar");


.. highlight:: text

From the terminal, run::

    python3 -m polar.parser --interactive policy.polar

We can now interactively query the Polar knowledge base.
For example, the query::

    >> allow(actor, action, resource)

will display the bindings for the ``actor``, ``action``, and ``resource``
variables that make the query ``True``::

    resource = "bar"
    actor = "foo"
    action = "read"

.. highlight:: polar
.. _inline-queries:

Inline queries
--------------
Queries can also be added to Polar files and will run when the file is loaded.
Failed/false queries will prevent the REPL from launching, or the file from
loading, but nothing is printed for successful queries. To add an inline query
to a Polar file, use the ``?=`` operator::

    # policy.polar
    ?= allow("foo", "read", "bar")

Inline queries are particularly useful for testing policies.
