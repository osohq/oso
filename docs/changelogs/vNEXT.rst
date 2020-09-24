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

``Oso.clear()`` replaced with ``Oso.clear_rules()``/``clearRules()``
---------------------------------------------------------------------

The ``Oso.clear()`` method in oso's language libraries has been removed.
To clear rules from the Polar knowledge base, use the new ``clear_rules()``
(or ``clearRules()``) method, which clears rules but leaves registered classes
and constants in place.

To migrate, replace calls to ``Oso.clear()`` with either ``Oso.clear_rules()`` or
``Oso.clearRules()``, depending on the library you are using.
It is no longer necessary to re-register classes/constants after clearing.

New features
============

Feature 1
---------

- summary
- of
- user facing changes

Link to relevant documentation section


Other bugs & improvements
=========================

- Language libraries that haven't yet implemented operations on application
  instances (Java, Node.js, Ruby, Rust) now throw a uniform error type.
