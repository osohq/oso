============================
Rust Authorization Library
============================

oso is packaged as a :doc:`cargo</download>` package for use in rust applications.

.. toctree::
    :hidden:

    /rust/index

To install, see :doc:`installation instructions </download>`.

Working with Rust Types
===========================

oso's Rust authorization library allows you to write policy rules over rust types directly.
This document explains how different rust types can be used in oso policies.

.. note::
  More detailed examples of working with application objects can be found in :doc:`/using/examples/index`.

Structs
^^^^^^^^^^^^^^^^
Rust struct types can be registered with oso which lets you pass them in and access their methods and fields. (see :ref:`application-types`).

Rust structs can also be constructed from inside an oso policy using the :ref:`operator-new` operator if the type has been given a constructor when registered.

Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^
Polar supports both integer and floating point numbers, as well as booleans (see :ref:`basic-types`).

Strings
^^^^^^^
Rust strings are mapped to Polar :ref:`strings`. Many of rust's string methods may be called in policies:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.username.ends_with("example.com");

.. code-block:: rust
  :caption: :fas:`rust` main.rs

  #[derive(PolarClass)]
  struct User {
    #[polar(attribute)]
    pub username: String
  }

  oso.register_class(User::get_polar_class()).unwrap();

  let user = User{username: "alice@example.com".to_owned()};
  assert!(oso.is_allowed(user, "foo", "bar");

.. warning::
  Polar does not support methods that mutate strings in place.

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
  * - Vec
    - List
  * - HashMap
    - Dictionary
