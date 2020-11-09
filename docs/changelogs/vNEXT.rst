====
NEXT
====

**Release date:** XXXX-XX-XX

Breaking changes
================

.. TODO remove warning and replace with "None" if no breaking
   changes.

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

Explicit iterator syntax
------------------------

Calling methods and accessing attributes on application instances no longer
returns multiple values. Instead, to iterate over values of an application
instance the explicit ``in`` operator must be used.

**Old syntax**:

   amount = recipe.ingredients().quantity

**New syntax**:

  item in recipe.ingredients() and
  amount = item.quantity

With this change, any methods on the iterable instance itself can now be used::

  recipe.ingredients().total_cost()

The above would call the ``total_cost`` method on the iterable returned by
``recipe.ingredients()``.

New features
============

``nil``
-------

Polar now defines `a constant named ``nil``
<https://docs.osohq.com/getting-started/policies/application-types.html#nil>`_
whose value is an application-language-specific "null" value;
e.g., ``None`` in Python, ``nil`` in Ruby, ``null`` for Java & JS, etc.
Explicit comparisons with ``nil`` are particularly useful in the
context of application-language methods that may return ``None``, etc.

In the (still experimental) context of list filtering via partial
evaluation, ``nil`` is intended to map to ``NULL`` in SQL. For instance,
partially evaluating the Polar expression ``x = nil`` with respect
to ``x`` yields a constraint that is translated into a check like
``X IS NULL``.

Other bugs & improvements
=========================

- We now check fields in the case of a ``matches`` against a built-in type. E.g.:

  .. code-block:: polar

    2 matches Integer { numerator: 2, denominator: 1 }
