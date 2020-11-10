.. meta::
  :description: Learn how to use oso with Java to add authorization to your application.

============================
Java Authorization Library
============================

oso is available as a :doc:`package</download>` for use in Java applications.

Code-level documentation is :doc:`here</java/index>`.

.. toctree::
    :hidden:

    /java/index

To install, see :doc:`installation instructions </download>`.

Working with Java Types
=======================

oso's Java authorization library lets you write policy rules over Java objects directly.
This document explains how different types of Java objects can be used in oso policies.


.. TODO: make below note reference correct doc

.. note::
    More detailed examples of working with application classes can be found in :doc:`/using/examples/index`.

Class Instances
^^^^^^^^^^^^^^^^
You may pass an instance of any Java class into oso and access its methods
and fields from your policy; see :ref:`application-types`.

Java instances can be constructed from within an oso policy using the
:ref:`operator-new` operator:

.. code-block:: polar
    :caption: :fa:`oso` policy.polar

    new User("alice@example.com")

To construct instances of a Java class, the class must be **registered**
using the ``registerClass()`` method:

.. code-block:: Java
    :caption: :fab:`java` User.java

    registerClass(User.class)

If you want to refer to the class using another name from within a policy,
you may supply an alias:

.. code-block:: Java
    :caption: :fab:`java` User.java

    registerClass(Person.class, "User")

At instantiation time, oso will search the list returned by
`Class.getConstructors()
<https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html#getConstructors()>`_
for a constructor that is applicable to the supplied positional constructor
arguments. For example, given the Polar expression ``new
User("alice@example.com")``, oso will search for a ``Constructor`` with one
parameter compatible with ``String.class``, e.g.:

.. code-block:: java
    :caption: :fab:`java` User.java

    public User(String username) { ... }

Applicability is determined using `Class.isAssignableFrom(Class<?> cls)
<https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html#isAssignableFrom(java.lang.Class)>`_,
which allows arguments that are instances of subclasses or implementations
of interfaces to properly match the constructor's parameter types.

.. TODO: link to javadoc above

Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^
Polar supports integer and floating point real numbers, as well as booleans (see :ref:`basic-types`).

.. note::
   Java primitives may be passed into oso, but numbers and booleans created in an oso policy will be
   converted to `autoboxed <https://docs.oracle.com/javase/tutorial/java/data/autoboxing.html>`_ `Integer`, `Float`, and `Boolean` types respectively.

   This means that methods called from oso must have autoboxed argument types. E.g.,

   .. code-block:: java
      :caption: :fab:`java` Foo.java

      class Foo {
         public static unboxed(int a, int b) {
            // ...
         }
         public static boxed(Integer a, Integer b) {
            // ...
         }
      }

   The ``boxed()`` method may be called from a policy,
   but attempting to call ``unboxed()`` will fail.

Strings
^^^^^^^
Java Strings are mapped to Polar :ref:`strings`. Java's String methods may be accessed from policies:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.username.endsWith("example.com");

.. code-block:: java
   :caption: :fab:`java` User.java

   public class User {
      public String username;

      public User(String username) {
         this.username = username;
      }

      public static void main(String[] args) {
         User user = new User("alice@example.com");
         assert oso.isAllowed(user, "foo", "bar");
      }
   }

Lists and Arrays
^^^^^^^^^^^^^^^^
Java `Arrays <https://docs.oracle.com/javase/tutorial/java/nutsandbolts/arrays.html>`_ *and* objects that implement the `List <https://docs.oracle.com/javase/10/docs/api/java/util/List.html>`_ interface are
mapped to Polar :ref:`Lists <lists>`. Java's ``List`` methods may be accessed from policies:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.groups.contains("HR");

.. code-block:: java
   :caption: :fab:`java` User.java

   public class User {
      public List<String> groups;

      public User(List<String> groups) {
         this.groups = groups;
      }

      public static void main(String[] args) {
         User user = new User(List.of("HR", "payroll"));
         assert oso.isAllowed(user, "foo", "bar");
      }
   }

Note that the ``isAllowed()`` call would also succeed if ``groups`` were an array.

.. warning::
    Polar does not support methods that mutate lists in place. E.g. ``add()`` will have no effect on
    a list in Polar.

Likewise, lists constructed in Polar may be passed into Java methods:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.has_groups(["HR", "payroll"]);

.. code-block:: java
   :caption: :fab:`java` User.java

      public boolean hasGroups(List<String> groups) {
         for(String g : groups) {
            if (!this.groups.contains(g))
               return false;
         }
         return true;
      }

      public static void main(String[] args) {
         User user = new User(List.of("HR", "payroll"));
         assert oso.isAllowed(user, "foo", "bar");
      }

.. |list_get| replace:: ``List.get``
.. _list_get: https://docs.oracle.com/javase/10/docs/api/java/util/List.html#get(int)

Java methods like |list_get|_ may be used for random access to
list elements, but there is currently no Polar syntax for that is
equivalent to the Java expression ``user.groups[1]``. To access
the elements of a list without using a method, you may iterate
over it with :ref:`operator-in` or destructure it with
:ref:`pattern matching <patterns-and-matching>`.

Maps
^^^^
Java objects that implement the `Map <https://docs.oracle.com/javase/10/docs/api/java/util/Map.html>`_ interface
are mapped to Polar :ref:`dictionaries`:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.roles.project1 = "admin";

.. code-block:: java
   :caption: :fab:`java` User.java

   public class User {
      public Map<String, String> roles;

      public User(Map<String, String> roles) {
         this.roles = roles;
      }

      public static void main(String[] args) {
         User user = new User(Map.of("project1", "admin"));
         assert oso.isAllowed(user, "foo", "bar");
      }
   }

Likewise, dictionaries constructed in Polar may be passed into Java methods.

Enumerations
^^^^^^^^^^^^
You may iterate over a Java `Enumeration <https://docs.oracle.com/javase/10/docs/api/java/util/Enumeration.html>`_
(or anything that can be converted to one, such as a ``Collection`` or ``Iterable``)
using the Polar :ref:`operator-in` operator:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if "payroll" in actor.getGroups();

.. code-block:: java
   :caption: :fab:`java` User.java

      public List<String> getGroups() {
         return List.of("HR", "payroll");
      }

      public static void main(String[] args) {
         User user = new User(Map.of("project1", "admin"));
         assert oso.isAllowed(user, "foo", "bar");
      }

``null``
^^^^^^^^
The Java ``null`` reference is registered as the Polar constant :ref:`nil`.
If a Java method can return ``null``, you may want to compare the result
to ``nil``:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.getOptional() != nil;

.. code-block:: ruby
  :caption: :fab:`java` User.java

  public class User {
      ...

      public Thing getOptional() {
          if someCondition() {
              return new Thing();
          } else {
              return null;
          }
      }
   }

Summary
^^^^^^^

.. list-table:: Java → Polar Types Summary
   :width: 500 px
   :header-rows: 1

   * - Java type
     - Polar type
   * - int/Integer
     - Integer
   * - float/Float
     - Float
   * - double/Double
     - Float
   * - boolean/Boolean
     - Boolean
   * - List
     - List
   * - Array
     - List
   * - Map
     - Dictionary
   * - String
     - String
