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

Previously, writing ``x.foo`` in Polar could be either accessing the attribute OR the
method called ``foo`` on ``x`` - the host language libraries would pick whichever it
found.

This syntax is no longer supporting. Methods can only be called by writing ``x.foo()``.


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