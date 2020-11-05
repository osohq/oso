.. meta::
  :description: Learn how to use oso with Rust to add authorization to your application.

============================
Rust Authorization Library
============================

oso is packaged as a :doc:`cargo</download>` crate for use in Rust applications.

API documentation for the crate lives `on docs.rs <https://docs.rs/oso/>`_.


.. toctree::
    :hidden:

To install, see :doc:`installation instructions </download>`.

Working with Rust Types
===========================

oso's Rust authorization library allows you to write policy rules over Rust types directly.
This document explains how different Rust types can be used in oso policies.

.. note::
  More detailed examples of working with application objects can be found in :doc:`/using/examples/index`.

Structs + Enums
^^^^^^^^^^^^^^^

Rust structs and enums can be registered with oso which lets you pass them in and access their methods and fields. (see :ref:`application-types`).

Rust structs can also be constructed from inside an oso policy using the :ref:`operator-new` operator if the type has been given a constructor when registered.

Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^
Polar supports both integer and floating point numbers, as well as booleans (see :ref:`basic-types`).

Strings
^^^^^^^
Rust `Strings <https://doc.rust-lang.org/std/string/struct.String.html>`_ are mapped to Polar :ref:`strings`. Many of rust's string methods may be called in policies:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.username.ends_with("example.com");

.. code-block:: rust
  :caption: :fab:`rust` main.rs

  #[derive(Clone, PolarClass)]
  struct User {
    #[polar(attribute)]
    pub username: String
  }

  oso.register_class(User::get_polar_class())?;

  let user = User{username: "alice@example.com".to_owned()};
  assert!(oso.is_allowed(user, "foo", "bar")?);

.. warning::
  Polar does not support methods that mutate strings in place.


Vectors
^^^^^^^
`Vec<T> <https://doc.rust-lang.org/std/vec/struct.Vec.html>`_ is mapped to Polar :ref:`Lists <lists>`, given that ``T: ToPolar``. 

Currently, no methods on ``Vec`` are exposed to Polar.

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if "HR" in actor.groups;

.. code-block:: rust
  :caption: :fab:`rust` main.rs

  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attribute)]
      pub groups: Vec<String>,
  }

  oso.register_class(User::get_polar_class())?;

  let user = User { groups: vec!["HR".to_string(), "payroll".to_string()] };
  assert!(oso.is_allowed(user, "foo", "bar")?);

.. warning::
  Polar does not support methods that mutate lists in place, unless the list is also returned from the method.


HashMaps
^^^^^^^^ 

Rust `HashMaps <https://doc.rust-lang.org/std/collections/struct.HashMap.html>`_ are mapped to Polar :ref:`dictionaries`,
but require that the ``HashMap`` key is a ``String``:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.roles.project1 = "admin";

.. code-block:: rust
  :caption: :fab:`rust` main.rs

  #[derive(Clone, PolarClass)]
  struct User {
      #[polar(attribute)]
      pub roles: HashMap<String, String>,
  }

  oso.register_class(User::get_polar_class())?;

  let user = User { roles: maplit::hashmap!{ "project1".to_string() => "admin".to_string() } };
  assert!(oso.is_allowed(user, "foo", "bar")?);

Likewise, dictionaries constructed in Polar may be passed into Ruby methods.

Iterators
^^^^^^^^^

oso handles Rust `iterators <https://doc.rust-lang.org/std/iter/index.html>`_ by evaluating the
yielded values one at a time. To register methods returning iterators, you need to use the
``Class::add_iterator_method`` call.

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.get_group() = "payroll";

.. code-block:: rust
  :caption: :fab:`rust` main.rs

    #[derive(Clone, PolarClass)]
    struct User {
        groups: Vec<String>,
    }

    oso.register_class(
        User::get_polar_class_builder()
            .add_iterator_method("get_group", |u: &User| u.groups.clone().into_iter())
            .build(),
    )
    .unwrap();

    let user = User {
        groups: vec!["HR".to_string(), "payroll".to_string()],
    };
    assert!(oso.is_allowed(user, "foo", "bar")?);

In the policy above, the body of the `allow` rule will first evaluate ``"HR" = "payroll"`` and then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to ``oso.is_allowed`` will succeed.
Note that if ``get_group`` returned an array instead of an iterator, the rule would fail because it would be comparing an array (``["HR", "payroll"]``) against a string (``"payroll"``).


Summary
^^^^^^^

.. list-table:: Rust -> Polar Types Summary
  :width: 500 px
  :header-rows: 1

  * - Rust type
    - Polar type
  * - i32, i64, usize
    - Number (Integer)
  * - f32, f64
    - Number (Float)
  * - bool
    - Boolean
  * - String, &'static str, str
    - String
  * - Vec
    - List
  * - HashMap
    - Dictionary
