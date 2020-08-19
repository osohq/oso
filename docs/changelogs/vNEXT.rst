=====
NEXT
=====

**Release date:** XXXX-XX-XX

Breaking changes
================

.. warning:: This release contains breaking changes. Be sure
   to follow migration steps before upgrading.

Method/Attribute syntax
-----------------------

Previously, ``x.foo`` and ``x.foo()`` in a Polar policy could either be
performing an attribute lookup or invoking a zero-arity method on ``x``. If
looking up the ``foo`` property returned a method, the host language libraries
would transparently invoke it and return the result.

**As of this release, parentheses are required for invocation**. ``x.foo``
performs a lookup, and ``x.foo()`` invokes a zero-arity method.

New features
==============

Feature 1
=========

- summary
- of
- user facing changes

Link to relevant documentation section

Other bugs & improvements
=========================

- Improved performance of policies with many rules having ground (constant) parameters.
- Improved performance of ``in`` operator (list membership) with many ground elements.
- Stack traces return the original policy source instead of the internal version.
- New ffi methods for passing printing and warning messages from rust to app languages.
