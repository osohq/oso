.. oso documentation master file, created by
   sphinx-quickstart on Fri Mar 20 10:34:51 2020.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

oso
===
**oso** is an authorization system designed to make it easy for developers to express
complex authorization logic naturally and concisely throughout an application.


Why oso
-------
1. oso makes it easy to build, debug, and maintain your authorization code
   using a declarative policy language, called Polar.
   Polar allows you to express a wide variety of authorization patterns – 
   including roles, attributes, hierarchies, integrations with identity providers,
   and others – naturally and concisely.
2. oso enables you to write rules directly over your application objects
   because it is tightly integrated with your application.
3. oso gives you the flexibility to write a policy in Polar and apply it to
   multiple applications or shared services at once, including across
   applications and services written in different languages.

Example
------

Let's look at a quick example of a Polar policy for an expense management application:

.. code-block:: polar

  # An owner of a resource can always read it.
  allow(user, "read", resource) := is_owner(user, resource);

  # Ownership is defined by resource properties.
  is_owner(user: User, budget: Budget) := user.id = budget.owner_id;
  is_owner(user: User, expense: Expense) := user.id = expense.owner_id;

  # Ownership is hierarchical across expenses and budgets.
  allow(user, "read", expense: Expense) := is_owner(user, expense.budget);

  # Accountants can approve budgets and expenses.
  allow(user, "approve", Budget) := user.role = "accountant";
  allow(user, "approve", Expense) := user.role = "accountant";

This short policy encodes:

1. Ownership semantics. Anyone who is an owner of a resource can read the resource. The definition
   of ownership can vary depending on the resource type.
2. A hierarchical relationship between resources. Anyone who is the owner of a
   budget can read expenses associated with that budget.
3. Role-based access. Users that are accountants can approve budgets.

Many authorization systems force developers to choose one of these models – like just roles or hierarchies or ownership semantics – but Polar's flexibility shines here, allowing the developer to use any or all of them.

What it's like to use oso
-------------------------
- Express your policy as code using the declarative :doc:`Polar language </language/index>`.
- Maintain authorization across a variety of languages and environments with a cross-language
  :doc:`authorization library </application-library/index>`.
- :ref:`Use native application objects & data <application-types>` directly in Polar policy.
- Understand why policy decisions are made using the :doc:`Policy debugger </dev-tools/debugger>`.
- :ref:`Write tests <testing>` over your policy to ensure correct behavior.

.. we don't support this yet:
.. - Integrations with common web frameworks and ORMs.


User Guide
==========

These guides contain narrative documentation and examples.  This is a great
starting point if you are new to **oso**.

.. toctree::
   :maxdepth: 2

   get-started

   auth-fundamentals

   auth-models/index

   application-library/index




Language & API Reference
=============

Reference material for more experienced oso users, or those looking to dive deep
on a particular concept.

.. toctree::
   :maxdepth: 2

   language/index
   dev-tools/index
   api/index

Changelog
=========

Release history of the oso project.

.. toctree::
   :maxdepth: 2

   changelogs/index


Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
