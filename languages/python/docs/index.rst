.. meta::
  :description: Oso Python library API reference.

==========
Python API
==========

Classes
=======

.. autoclass:: oso.Oso
   :members: __init__, is_allowed, get_allowed_actions, load_str, load_file, load_files,
      register_class, query_rule, query, clear_rules, authorize,
      authorize_request, authorize_field, authorized_actions, authorized_fields,
      authorized_resources, authorized_query, set_data_filtering_query_defaults,
      register_constant, set_data_filtering_adapter

.. autoclass:: oso.Variable

.. autoclass:: oso.Predicate

.. autoclass:: polar.Relation

.. autoclass:: polar.Filter

.. autoclass:: polar.DataFilter

.. autoclass:: polar.Condition

.. autoclass:: polar.Projection

.. autoclass:: polar.data.adapter.sqlalchemy_adapter.SqlAlchemyAdapter


Exceptions
==========

.. automodule:: oso.exceptions
    :members:
    :show-inheritance:

.. automodule:: polar.exceptions
    :members:
    :show-inheritance:
