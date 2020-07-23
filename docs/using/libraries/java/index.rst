============================
Java Authorization Library
============================

oso is available as a :doc:`package</getting-started/download/java>` for use in Java applications.

Code-level documentation is :doc:`here</java/index>`.

.. toctree::
    :hidden:

    /java/index

To install, see :doc:`installation instructions </getting-started/download/java>`.

Working with Java Types
=======================

oso's Java authorization library allows you to write policy rules over Java objects directly.
This document explains how different types of Java objects can be used in oso policies.


.. TODO: make below note reference correct doc

.. note::
    More detailed examples of working with application classes can be found in :doc:`/using/examples/index`.

Class Instances
^^^^^^^^^^^^^^^^
You can pass an instance of any Java class into oso and access its methods and fields from your policy (see :ref:`application-types`).

Java instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Java class has been **registered** using
the ``registerClass()`` method. To register a class in Java, you must provide an implementation of ``Function<Map, Object>`` (see `Java's Function interface <https://docs.oracle.com/javase/8/docs/api/java/util/function/Function.html>`_).
An example of this can be found :ref:`here <application-types>`.

.. TODO: link to javadoc above


Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^
Polar supports both integer and floating point numbers, as well as booleans (see :ref:`basic-types`).

.. note::
   Java primitives may be passed into oso, but numbers and booleans created in an oso policy will be
   converted to `autoboxed <https://docs.oracle.com/javase/tutorial/java/data/autoboxing.html>`_ `Integer`, `Float`, and `Boolean` types respectively.

   This means that methods called from oso must have autoboxed argument types. E.g.,

   .. code-block:: java

      class Foo {
         public static method1(int a, int b) {
            // ...
         }
         public static method2(Integer a, Integer b) {
            // ...
         }
      }

   ``method2()`` above may be called from a policy file, however attempting to call ``method1()`` will fail.


Strings
^^^^^^^
Java Strings are mapped to Polar :ref:`strings`. Java's String methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.username.endsWith("example.com");

.. code-block:: java
   :caption: User.java

   public class User {
      public String username;

      public User(String username) {
         this.username = username;
      }

      public static void main(String[] args) {
         User user = new User("alice@example.com");
         assert oso.allow(user, "foo", "bar");
      }
   }

Lists and Arrays
^^^^^^^^^^^^^^^^
Java `Arrays <https://docs.oracle.com/javase/tutorial/java/nutsandbolts/arrays.html>`_ *and* objects that implement the `List <https://docs.oracle.com/javase/8/docs/api/java/util/List.html>`_ interface are
mapped to Polar :ref:`Lists <lists>`. Java's ``List`` methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.groups.contains("HR");

.. code-block:: java
   :caption: User.java

   public class User {
      public List<String> groups;

      public User(List<String> groups) {
         this.groups = groups;
      }

      public static void main(String[] args) {
         User user = new User(List.of("HR", "payroll"));
         assert oso.allow(user, "foo", "bar");
      }
   }

Note that the ``allow()`` call would also succeed if ``groups`` were an array.

.. warning::
    Polar does not support methods that mutate lists in place. E.g. ``add()`` will have no effect on
    a list in Polar.

Likewise, lists constructed in Polar may be passed into Java methods:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.has_groups(["HR", "payroll"]);

.. code-block:: java
   :caption: User.java

      public boolean hasGroups(List<String> groups) {
         for(String g : groups) {
            if (!this.groups.contains(g))
               return false;
         }
         return true;
      }

      public static void main(String[] args) {
         User user = new User(List.of("HR", "payroll"));
         assert oso.allow(user, "foo", "bar");
      }

Maps
^^^^
Java objects that implement the `Map <https://docs.oracle.com/javase/8/docs/api/java/util/Map.html>`_ interface
are mapped to Polar :ref:`dictionaries`:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.roles.project1 = "admin";

.. code-block:: java
   :caption: User.java

   public class User {
      public Map<String, String> roles;

      public User(Map<String, String> roles) {
         this.roles = roles;
      }

      public static void main(String[] args) {
         User user = new User(Map.of("project1", "admin"));
         assert oso.allow(user, "foo", "bar");
      }
   }

Likewise, dictionaries constructed in Polar may be passed into Java methods.

Enumerations
^^^^^^^^^^^^
Oso handles Java objects that implement the `Enumeration <https://docs.oracle.com/javase/7/docs/api/java/util/Enumeration.html>`_ interface by evaluating each of the
object's elements one at a time:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.getGroup = "payroll";

.. code-block:: java
   :caption: User.java

      public Enumeration<String> getGroup() {
         return Collections.enumeration(List.of("HR", "payroll"));
      }

      public static void main(String[] args) {
         User user = new User(Map.of("project1", "admin"));
         assert oso.allow(user, "foo", "bar");
      }

In the policy above, the right hand side of the `allow` rule will first evaluate ``"HR" = "payroll"``, then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to ``allow()`` will succeed.
Note that if ``getGroup()`` returned a list, the rule would fail, as the evaluation would be ``["HR", "payroll"] = "payroll"``.

Summary
^^^^^^^

.. list-table:: Java -> Polar Types Summary
   :widths: 500 500
   :header-rows: 1

   * - Java type
     - Polar type
   * - int/Integer
     - Number (Integer)
   * - float/Float
     - Number (Float)
   * - double/Double
     - Number (Float)
   * - boolean/Boolean
     - Boolean
   * - List
     - List
   * - Array
     - List
   * - Map
     - Dictionary
