==============
The Polar REPL
==============

Developers can query Polar knowledge bases from the command line using the Polar REPL.
To run the REPL, first make sure you have :ref:`installed oso <install>`.

Once oso is installed, we can launch the REPL from the terminal::

    python3 -m polar.parser --interactive <policy files>

.. Both Python and Polar files can be loaded into the REPL.

.. TODO: (leina) remove this once fix is merged
.. .. note::
..    Python files must be loaded into the REPL first if they define classes
..    referenced in the Polar files.


Let's start by loading a simple Polar policy::

    # policy.polar
    allow(actor, "read", resource) := owns(actor, resource);
    owns("foo", "bar");


.. highlight:: text

From the terminal, run::

    python3 -m polar.parser --interactive policy.polar

We can now interactively query the Polar knowledge base.

For example, the query::

    ?= allow(actor, action, resource)

will display the bindings for the ``actor``, ``action``, and ``resource`` variables that make
the query ``True``::

    resource = "bar"
    actor = "foo"
    action = "read"

.. highlight:: polar

Queries can also be added to Polar files and will run when the file is loaded. Failed/false queries will
prevent the REPL from launching. To add a query to a Polar file, use the ``?=`` operator::

    # policy.polar
    ?= allow("foo", "read", "bar")

