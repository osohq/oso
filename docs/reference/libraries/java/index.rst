============================
Java Authorization Library
============================

Oso currently provides an authorization library to integrate oso with Java applications.

Code-level documentation is :doc:`here</java/index>`.

Working with Java Types
=======================

oso's Java authorization library allows you to write policy rules over Java objects directly.
This document explains how different types of Java objects can be used in oso policies.

.. note::
    More detailed examples of working with application classes can be found in :ref:`auth-models`.

Class Instances
^^^^^^^^^^^^^^^^
You can pass an instance of any Java class into oso and access its methods and fields from your policy.

For example:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.isAdmin;

The above rule expects the ``actor`` variable to be a Java instance with the field ``isAdmin``.
The Java instance is passed into oso with a call to ``Oso.allow``:

.. TODO: add link to javadocs

.. code-block:: java
   :caption: User.java

   public class User {
      public boolean isAdmin;
      public String name;

      public User(String name, boolean isAdmin) {
         this.isAdmin = isAdmin;
         this.name = name;
      }

      public static void main(String[] args) {
         User user = new User("alice", true);
         assert oso.allow(user, "foo", "bar");
      }
   }


The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has a field
called ``isAdmin``, it is evaluated by the Polar rule and found to be true.

Java instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Java class has been **registered** using
the ``registerClass()`` method.
.. TODO: link to javadoc above

Registering classes also makes it possible to use :ref:`specialization` and the
:ref:`operator-matches` with the registered class:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) if actor matches User{name: "alice", isAdmin: true};

.. code-block:: java
   :caption: User.java

      public static void main(String[] args) {
         oso.registerClass(User, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

         User user = new User("alice", true);
         assert oso.allow(user, "foo", "bar");
         assert !oso.allow("notauser", "foo", "bar");
      }

Once a class is registered, its static methods can also be called from oso policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) if actor.name in User.superusers();

.. code-block:: java
   :caption: User.java

      public static List<String> superusers() {
         return List.of("alice", "bhavik", "clarice");
      }

      public static void main(String[] args) {
         oso.registerClass(User, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

         User user = new User("alice", true);
         assert oso.allow(user, "foo", "bar");
      }


Numbers
^^^^^^^
Polar supports both integer and floating point numbers (see :ref:`basic-types`)

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


