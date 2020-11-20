.. meta::
  :description: Explore our guides and reference articles for learning oso and adding it to your application.

oso Documentation
=================

oso is an **open source policy engine for authorization** that's embedded in your
application. It provides a declarative policy language for expressing
authorization logic. You define this logic separately from the rest of your
application code, but it executes inside the application and can call directly
into it. oso ships as a library with a debugger and a REPL.

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



.. container:: cta

    **oso is now in Developer Preview**

    .. button::
        :link: /getting-started/quickstart
        :text: Try oso
        :class: get-started

    .. button::
        :link: /more/design-principles
        :text: Learn More
        :class: learn-more

.. toctree::
   :maxdepth: 1
   :caption: Getting Started
   :hidden:

   getting-started/quickstart
   getting-started/application/index
   Writing Policies <getting-started/policies/index>
   Role-Based Access Control <getting-started/rbac>
   List Filtering <getting-started/list-filtering/index>

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


.. todolist::
