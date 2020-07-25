.. oso documentation master file, created by
   sphinx-quickstart on Fri Mar 20 10:34:51 2020.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Welcome to the oso documentation!
=================================

oso is an **open source policy engine for authorization** that's embedded in your application.
It provides a declarative policy language for expressing authorization logic,
which you define separately from the rest of your application code but which executes inside the
application and can call directly into it.
oso ships as a library with a debugger and a REPL.

.. container:: left-col

    .. code-block:: python
        :caption: :fab:`python` app.py

        if oso.is_allowed(user, "view", expense):
            # ...

.. container:: right-col

    .. code-block:: polar
        :caption: :fa:`oso` expense.polar

        allow(user, "read", expense) if
            user.email = expense.submitted_by;

You write policies to define authorization logic. You tell it things like who should access what, based
on what you know about them and their relationship to what they are trying to access.
Policies are written in a declarative policy language called Polar, then they are loaded into oso.

Using oso, you can:

1. Separate authorization code from business logic
2. Express policies concisely with a declarative language
3. Write policies that call directly into your application

.. button::
    :link: /getting-started/quickstart
    :text: Get Started

.. button::
    :link: /more/design-principles
    :text: Learn More
    :class: matter-success
.. options: matter-primary, matter-secondary, matter-error, matter-warning, matter-success

-----

.. toctree::
    :maxdepth: 1
    :titlesonly:
    :hidden:

    Home <self>

.. These guides contain a brief introduction to oso. This is a great
.. starting point if you are new to **oso**.

.. toctree::
   :maxdepth: 1
   :caption: Getting Started
   :hidden:

   getting-started/quickstart
   getting-started/download/index
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
   more/internals
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
