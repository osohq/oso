.. oso documentation master file, created by
   sphinx-quickstart on Fri Mar 20 10:34:51 2020.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.


Welcome to the oso documentation!
==================================

oso helps developers build authorization in their applications.

.. admonition:: What is oso?

    oso is an open source policy engine for authorization that's embedded in your application. It provides a declarative policy language for expressing authorization logic, which you define separately from your application code but which executes inside the application and can call directly into it. oso ships as a library with a debugger and a REPL.

Using oso, you can:

1. Separate authorization code from business logic
2. Express policies concisely with a declarative language
3. Write policies that call directly into your application

To see this in action, :doc:`continue on to the Getting Started guide <getting-started/quickstart>`.

To learn more about oso and the principles behind its design,
:doc:`read the oso overview page <more/design-principles>`.

-----

.. toctree::
    :maxdepth: 1
    :titlesonly:
    :hidden:

    Docs Home <self>

.. These guides contain a brief introduction to oso. This is a great
.. starting point if you are new to **oso**.

.. toctree::
   :maxdepth: 1
   :caption: Getting Started
   :hidden:

   getting-started/download/index
   getting-started/quickstart
   getting-started/application/index
   Writing Policies <getting-started/policies/index>

.. ----------

.. These guides contain narrative documentation and examples.
.. This is a good place to go to learn more about implementing
.. authorization in your application with oso.

.. toctree::
   :maxdepth: 1
   :caption: Using oso
   :hidden:

   using/examples/index
   using/polar-syntax
   using/libraries/index

.. ----------

.. toctree::
   :maxdepth: 1
   :caption: More
   :titlesonly:
   :hidden:

   more/design-principles
   more/key-concepts
   more/dev-tools/index
   more/faq
   more/use-cases
   more/performance
   more/language/polar-foundations
   more/security
   project/changelogs/index

..
.. Indices and tables
.. ==================
..
.. * :ref:`genindex`
.. * :ref:`modindex`
.. * :ref:`search`



.. ifconfig:: todo_include_todos

    (Internal) List of TODOS
    ------------------------

.. todolist::
