.. meta::
  :description: Reference page on oso's underlying integration and Python API.

==========
Python API
==========

Classes
=======

.. autoclass:: oso.Oso
   :members: is_allowed, load_str, load_file, register_class, query_rule, query, clear_rules, get_allowed_actions

.. autoclass:: oso.Variable

.. autoclass:: oso.Predicate

Decorator Functions
^^^^^^^^^^^^^^^^^^^
.. autofunction:: oso.polar_class

Exceptions
==========

.. automodule:: polar.exceptions
    :members:
    :show-inheritance:
