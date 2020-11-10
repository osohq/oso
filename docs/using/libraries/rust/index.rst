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
Polar supports integer and floating point real numbers, as well as booleans (see :ref:`basic-types`).

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

.. |vec_get| replace:: ``Vec::get``
.. _vec_get: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.get

There is currently no syntax for random access to a vector element within a
policy; i.e., there is no Polar equivalent of the JavaScript expression
``user.groups[1]``. However, you can still access a specific element via
methods like |vec_get|_. To access the elements of a list, you may iterate
over it with :ref:`operator-in` or destructure it with :ref:`pattern matching
<patterns-and-matching>`.

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

You may iterate over a Rust `iterator <https://doc.rust-lang.org/std/iter/index.html>`_
using the Polar :ref:`operator-in` operator:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if "payroll" in actor.get_groups();

.. code-block:: rust
  :caption: :fab:`rust` main.rs

    #[derive(Clone, PolarClass)]
    struct User {
        groups: Vec<String>,
    }

    oso.register_class(
        User::get_polar_class_builder()
            .add_iterator_method("get_groups", |u: &User| u.groups.clone().into_iter())
            .build(),
    )
    .unwrap();

    let user = User {
        groups: vec!["HR".to_string(), "payroll".to_string()],
    };
    assert!(oso.is_allowed(user, "foo", "bar")?);

Summary
^^^^^^^

.. list-table:: Rust → Polar Types Summary
  :width: 500 px
  :header-rows: 1

  * - Rust type
    - Polar type
  * - i32, i64, usize
    - Integer
  * - f32, f64
    - Float
  * - bool
    - Boolean
  * - Vec
    - List
  * - HashMap
    - Dictionary
  * - String, &'static str, str
    - String
