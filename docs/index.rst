oso Documentation
=================

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

You write policies to define authorization logic. You tell oso things like who
should access what, based on what you know about them and their relationship to
what they are trying to access. Policies are written in a declarative policy
language called Polar, then they are loaded into oso.

Using oso, you can:

1. Separate authorization code from business logic, but keep
data where it is
2. Express policies concisely with a declarative language
3. Start from simple building blocks, then extend the system
as needed

.. button::
    :link: /getting-started/quickstart
    :text: Get Started

.. button::
    :link: /more/design-principles
    :text: Learn More
    :class: matter-success
.. options: matter-primary, matter-secondary, matter-error, matter-warning, matter-success

.. toctree::
   :maxdepth: 1
   :caption: Getting Started
   :hidden:

   getting-started/quickstart
   getting-started/application/index
   Writing Policies <getting-started/policies/index>

.. toctree::
   :maxdepth: 1
   :caption: Using oso
   :hidden:

   using/examples/index
   using/polar-syntax
   using/libraries/index
   more/dev-tools/index

.. toctree::
   :maxdepth: 1
   :caption: More
   :titlesonly:
   :hidden:

   more/design-principles
   more/glossary
   more/faq
   more/use-cases
   more/performance/index
   more/security
   more/language/polar-foundations
   more/internals


.. ifconfig:: todo_include_todos

    (Internal) List of TODOS
    ------------------------

.. todolist::
