
.. JAVA EXAMPLES

=================
Application types
=================

Any type defined in an application can be passed into oso, and its
attributes may be accessed from within a policy. Using application types
make it possible to take advantage of an app's existing domain model. For example:

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            allow(actor, action, resource) if actor.is_admin;

        The above rule expects the ``actor`` variable to be a Python instance with the attribute ``is_admin``.
        The Python instance is passed into oso with a call to :py:meth:`~oso.Oso.allow`:

        .. code-block:: python
            :caption: :fab:`python` app.py

            class User:
                def __init__(self, name, is_admin):
                    self.name = name
                    self.is_admin = is_admin

            user = User("alice", True)
            assert(oso.is_allowed(user, "foo", "bar))

        The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
        called ``is_admin``, it is evaluated by the policy and found to be true.

    .. group-tab:: Ruby

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            allow(actor, action, resource) if actor.is_admin;

        The above rule expects the ``actor`` variable to be a Ruby instance with the attribute ``is_admin``.
        The Ruby instance is passed into oso with a call to ``Oso#allowed?``:

        .. code-block:: ruby
            :caption: :fas:`gem` app.rb

            class User
                attr_reader :name
                attr_reader :is_admin

                def initialize(name, is_admin)
                    @name = name
                    @is_admin = is_admin
                end
            end

            user = User.new("alice", true)
            raise "should be allowed" unless OSO.allowed?(user, "foo", "bar")

        The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
        called ``is_admin``, it is evaluated by the policy and found to be true.

    .. group-tab:: Java

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            allow(actor, action, resource) if actor.isAdmin;

        The above rule expects the ``actor`` variable to be a Java instance with the field ``isAdmin``.
        The Java instance is passed into oso with a call to ``Oso.allow``:

        .. code-block:: java
            :caption: :fab:`java` User.java

            public class User {
                public boolean isAdmin;
                public String name;

                public User(String name, boolean isAdmin) {
                    this.isAdmin = isAdmin;
                    this.name = name;
                }

                public static void main(String[] args) {
                    User user = new User("alice", true);
                    assert oso.isAllowed(user, "foo", "bar");
                }
            }

        The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has a field
        called ``isAdmin``, it is evaluated by the Polar rule and found to be true.

.. note::
    You can also call methods on application instances in a policy. If the method takes arguments, the method must be called
    with `ordered arguments`, even if the method is defined to take keyword arguments.



Registering Application Types
==============================

Instances of application types can be constructed from inside an oso policy using the :ref:`operator-new` operator if the class has been **registered**:

.. tabs::
    .. group-tab:: Python
        We can register a Python class using :py:meth:`oso.Oso.register_class` or the :py:func:`~oso.polar_class` decorator:

        .. code-block:: python
            :caption: :fab:`python` app.py

            oso.register_class(User)

        Once the class is registered, we can make a ``User`` object in Polar. This can be helpful for writing inline queries:

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            ?= allow(new User{name: "alice", is_admin: true}, "foo", "bar");

    .. group-tab:: Ruby
        Ruby classes are registered using ``register_class()``(see :doc:`/ruby/index`):

        .. code-block:: ruby
            :caption: :fas:`gem` app.rb

            OSO.register_class(User)

        Once the class is registered, we can make a ``User`` object in Polar. This can be helpful for writing inline queries:

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            ?= allow(new User{name: "alice", is_admin: true}, "foo", "bar");

    .. group-tab:: Java
        To register a Java class, you must provide a lambda function to ``registerClass()`` that takes a map of arguments:

        .. code-block:: java
            :caption: :fab:`java` User.java

            public static void main(String[] args) {
                oso.registerClass(User.class, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");
            }

        Once the class is registered, we can make a ``User`` object in Polar. This can be helpful for writing inline queries:

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            ?= allow(new User{name: "alice", isAdmin: true}, "foo", "bar");



Registering classes also makes it possible to use :ref:`specialization` and the :ref:`operator-matches` with the registered class.

In our previous example, the **allow** rule expected the actor to be a ``User``, but we couldn't actually check
that type assumption in the policy. If we register the ``User`` class, we can write the following rule:

.. code-block:: polar
    :caption: :fa:`oso` policy.polar

    allow(actor: User, action, resource) if actor.name = "alice";


This rule will only be evaluated when the actor is a ``User``.
We could also use ``matches`` to express the same logic:

.. code-block:: polar
    :caption: :fa:`oso` policy.polar

    allow(actor, action, resource) if matches User{name: "alice"};

.. tabs::
    .. group-tab:: Python

        We can then evaluate the rule:

        .. code-block:: python
            :caption: :fab:`python` app.py

            oso.register_class(User)

            user = User("alice", True)
            assert(oso.is_allowed(user, "foo", "bar))
            assert(not oso.is_allowed("notauser", "foo", "bar"))

    .. group-tab:: Ruby

        We can then evaluate the rule:

        .. code-block:: ruby
            :caption: :fas:`gem` app.rb

            OSO.register_class(User)
            user = User.new("alice", true)
            raise "should be allowed" unless OSO.allowed?(user, "foo", "bar")
            raise "should not be allowed" unless not OSO.allowed?(user, "foo", "bar")

    .. group-tab:: Java

        We can then evaluate the rule:

        .. code-block:: java
            :caption: :fab:`java` User.java

            public static void main(String[] args) {
                oso.registerClass(User.class, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

                User user = new User("alice", true);
                assert oso.isAllowed(user, "foo", "bar");
                assert !oso.isAllowed("notauser", "foo", "bar");
            }


.. note::
    Type specializers automatically respect the
    **inheritance** hierarchy of our application classes. See our :doc:`/using/examples/inheritance` guide for an
    in-depth example of how this works.

Once a class is registered, its static methods can also be called from oso policies:

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            allow(actor: User, action, resource) if actor.name in User.superusers();

        .. code-block:: python
            :caption: :fab:`python` app.py

            class User:
                ...
                @classmethod
                def superusers(cls):
                    """ Class method to return list of superusers. """
                    return ["alice", "bhavik", "clarice"]

            oso.register_class(User)

            user = User("alice", True)
            assert(oso.is_allowed(user, "foo", "bar))

    .. group-tab:: Ruby

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            allow(actor: User, action, resource) if actor.name in User.superusers();

        .. code-block:: ruby
            :caption: :fas:`gem` app.rb

            class User
                # ...
                def self.superusers
                    ["alice", "bhavik", "clarice"]
                end
            end

            OSO.register_class(User)

        user = User.new("alice", true)
        raise "should be allowed" unless OSO.allowed?(user, "foo", "bar")

    .. group-tab:: Java

        .. code-block:: polar
            :caption: :fa:`oso` policy.polar

            allow(actor: User, action, resource) if actor.name in User.superusers();

        .. code-block:: java
            :caption: :fab:`java` User.java

            public static List<String> superusers() {
                return List.of("alice", "bhavik", "clarice");
            }

            public static void main(String[] args) {
                oso.registerClass(User.class, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

                User user = new User("alice", true);
                assert oso.isAllowed(user, "foo", "bar");
            }

Built-in types
==============

Methods called on Polar built-ins (``str``, ``dict``, ``number`` & ``list``)
call methods on the corresponding language type. That way you can use
familiar methods like ``str.startswith()`` on strings regardless of whether
they originated in your application or as a literal in your policy.
This applies to all of the Polar :ref:`supported types <basic-types>`:
strings, lists, dictionaries, and numbers, in any supported application
language. For examples using built-in types, see the :doc:`/using/libraries/index` guides.

.. warning:: Do not attempt to mutate a literal using a method on it.
  Literals in Polar are constant, and any changes made to such objects
  by calling a method will not be persisted.


Summary
=======

* **Application types** and their associated application data are available
  within policies.
* Types can be **registered** with oso, in order to:
    * Create instances of application types in policies
    * Leverage the inheritance structure of application types with **specialized
      rules**, supporting more sophisticated access control models.
* You can use **built-in methods** on primitive types & literals like strings
  and dictionaries, exactly as if they were application types.

.. admonition:: What's next
    :class: tip

    * Explore how to implement common authorization models in oso, like
      **role-based** and **attribute-based access control**:
      :doc:`/using/examples/index`.
    * Learn more about using application types with your language in:
      :doc:`/using/libraries/index`.
