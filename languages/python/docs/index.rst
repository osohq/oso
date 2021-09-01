.. meta::
  :description: oso Python library API reference.

==========
Python API
==========

Classes
=======

.. autoclass:: oso.Oso
   :members: __init__, is_allowed, get_allowed_actions, load_str, load_file,
      register_class, query_rule, query, clear_rules, authorize,
      authorize_request, authorize_field, authorized_actions, authorized_fields

.. autoclass:: oso.Variable

.. autoclass:: oso.Predicate

Decorator Functions
^^^^^^^^^^^^^^^^^^^
.. autofunction:: oso.polar_class

Exceptions
==========

.. automodule:: oso.exceptions
    :members:
    :show-inheritance:

.. automodule:: polar.exceptions
    :members:
    :show-inheritance:
