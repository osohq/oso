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

You may also register a Java class with a particular `Constructor
<https://docs.oracle.com/javase/10/docs/api/java/lang/reflect/Constructor.html>`_
(obtained via, e.g., `Class.getConstructor(Class...)
<https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html#getConstructor(java.lang.Class...)>`_):

.. code-block:: Java
    :caption: :fab:`java` User.java

    registerClass(User.class, User.class.getConstructor(...))

If you omit the constructor (recommended), the default behavior at instantiation
time is to search in the list returned by `Class.getConstructors()
<https://docs.oracle.com/javase/10/docs/api/java/lang/Class.html#getConstructors()>`_
for a constructor that is applicable to the supplied positional constructor
arguments. For example, given the Polar expression ``new User("alice@example.com")``,
oso will search for a ``Constructor`` with one parameter compatible with
``String.class``, e.g.:

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
Polar supports both integer and floating point numbers, as well as booleans (see :ref:`basic-types`).

.. note::
   Java primitives may be passed into oso, but numbers and booleans created in an oso policy will be
   converted to `autoboxed <https://docs.oracle.com/javase/tutorial/java/data/autoboxing.html>`_ `Integer`, `Float`, and `Boolean` types respectively.

   This means that methods called from oso must have autoboxed argument types. E.g.,

   .. code-block:: java
      :caption: :fab:`java` Foo.java

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
Oso handles Java objects that implement the `Enumeration <https://docs.oracle.com/javase/10/docs/api/java/util/Enumeration.html>`_ interface by evaluating each of the
object's elements one at a time:

.. code-block:: polar
   :caption: :fa:`oso` policy.polar

   allow(actor, action, resource) if actor.getGroup = "payroll";

.. code-block:: java
   :caption: :fab:`java` User.java

      public Enumeration<String> getGroup() {
         return Collections.enumeration(List.of("HR", "payroll"));
      }

      public static void main(String[] args) {
         User user = new User(Map.of("project1", "admin"));
         assert oso.isAllowed(user, "foo", "bar");
      }

In the policy above, the right hand side of the `allow` rule will first evaluate ``"HR" = "payroll"``, then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to ``isAllowed()`` will succeed.
Note that if ``getGroup()`` returned a list, the rule would fail, as the evaluation would be ``["HR", "payroll"] = "payroll"``.

Summary
^^^^^^^

.. list-table:: Java -> Polar Types Summary
   :width: 500 px
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
