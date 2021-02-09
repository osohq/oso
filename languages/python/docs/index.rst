.. meta::
  :description: oso Python library API reference.

==========
Python API
==========

Classes
=======

.. autoclass:: oso.Oso
   :members: is_allowed, get_allowed_actions, load_str, load_file, register_class, query_rule, query, clear_rules

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
