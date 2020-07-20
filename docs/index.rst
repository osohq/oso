.. oso documentation master file, created by
   sphinx-quickstart on Fri Mar 20 10:34:51 2020.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.


Welcome to the oso documentation!
==================================

.. todo::
    "what is oso" wording 

.. admonition:: What is oso?

    oso is a library for adding authorization to applications using a declarative
    policy language

The core use case of oso is to add authorization logic to any application.
This is commonly solved by custom logic sprinkled ad hoc throughout an application,
leading to code that is hard to maintain, modify, and debug.

By using oso, you can:

1. Separate authorization code from business logic
2. Concisely express policies with a declarative language
3. Interact directly with your application data

:guilabel:`testing`

To see this in action, :doc:`continue on to the Getting Started guide <getting-started/quickstart>`.

To learn more about oso and the principles behind its design, 
:doc:`read the oso overview page <understand/overview>`.

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

   getting-started/quickstart
   getting-started/download

.. ----------

.. These guides contain narrative documentation and examples.
.. This is a good place to go to learn more about implementing
.. authorization in your application with oso.

.. toctree::
   :maxdepth: 1
   :caption: Understand oso
   :hidden:

   understand/overview
   understand/auth-fundamentals

   understand/application/index
   understand/policies/index

.. ----------
.. 
.. Reference material for concepts, terminology and language APIs.

.. toctree::
   :maxdepth: 1
   :caption: Reference
   :titlesonly:
   :hidden:

   reference/libraries/index
   reference/frameworks/index
   reference/polar-syntax
   reference/dev-tools/index

   reference/performance
   reference/faq

..  ----------
..  
..  Project-related information.

.. todo::
   Add links to Github + Issues

.. toctree::
   :maxdepth: 1
   :caption: Project
   :titlesonly:
   :hidden:


   project/changelogs/index
   Github <https://github.com/osohq/oso>

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
