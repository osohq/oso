
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
            :caption: policy.polar

            allow(actor, action, resource) if actor.is_admin;

        The above rule expects the ``actor`` variable to be a Python instance with the attribute ``is_admin``.
        The Python instance is passed into oso with a call to :py:meth:`~oso.Oso.allow`:

        .. code-block:: python
            :caption: app.py

            user = User()
            user.is_admin = True
            assert(OSO.allow(user, "foo", "bar))

        The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
        called ``is_admin``, it is evaluated by the policy and found to be true.

    .. group-tab:: Ruby

        .. code-block:: polar
            :caption: policy.polar

            allow(actor, action, resource) if actor.is_admin;

        The above rule expects the ``actor`` variable to be a Ruby instance with the attribute ``is_admin``.
        The Ruby instance is passed into oso with a call to ``allow()``:

        .. code-block:: ruby
            :caption: app.rb

            class User
                attr_reader :name
                attr_reader :is_admin

                def initialize(name, is_admin)
                    @name = name
                    @is_admin = is_admin
                end
            end

            user = User.new("alice", true)
            raise "should be allowed" unless OSO.allow(user, "foo", "bar")

        The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
        called ``is_admin``, it is evaluated by the policy and found to be true.

    .. group-tab:: Java

        .. code-block:: polar
            :caption: policy.polar

            allow(actor, action, resource) if actor.isAdmin;

        The above rule expects the ``actor`` variable to be a Java instance with the field ``isAdmin``.
        The Java instance is passed into oso with a call to ``Oso.allow``:

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
                    assert OSO.allow(user, "foo", "bar");
                }
            }

        The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has a field
        called ``isAdmin``, it is evaluated by the Polar rule and found to be true.



Registering Application Types
==============================

Instances of application types can be constructed from inside an oso policy using the :ref:`operator-new` operator if the class has been **registered**.
Registering classes also makes it possible to use :ref:`specialization` and the :ref:`operator-matches` with the registered class.

In our previous example, the **allow** rule expected the actor to be a ``User``, but couldn't actually check
that type assumption in the policy. If we register the ``User`` class, we can write the following rule:

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor matches User{name: "alice"};

        This rule will only be evaluated when the actor is a ``User``. We're also able to use ``matches`` on the actor.

        We can register the class using :py:meth:`oso.Oso.register_class` or the :py:func:`~oso.polar_class` decorator,
        and then evaluate the rule:

        .. code-block:: python
            :caption: app.py

            oso.register_class(User)

            user = User()
            user.name = "alice"
            assert(oso.allow(user, "foo", "bar))
            assert(not oso.allow("notauser", "foo", "bar"))

    .. group-tab:: Ruby

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor matches User{name: "alice", is_admin: true};

        This rule will only be evaluated when the actor is a ``User``. We're also able to use ``matches`` on the actor.

        We can register the class using ``register_class()``(see :doc:`/ruby/index`),
        and then evaluate the rule:

        .. code-block:: ruby
            :caption: app.rb

            OSO.register_class(User)
            user = User.new("alice", true)
            raise "should be allowed" unless OSO.allow(user, "foo", "bar")
            raise "should not be allowed" unless not OSO.allow(user, "foo", "bar")

    .. group-tab:: Java

        Classes in Java are registered using the ``Oso.registerClass()`` method:

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor matches User{name: "alice", isAdmin: true};

        This rule will only be evaluated when the actor is a ``User``. We're also able to use ``matches`` on the actor.

        We can register the class using ``registerClass()`` (see :doc:`/java/index`),
        and then evaluate the rule:

        .. code-block:: java
            :caption: User.java

            public static void main(String[] args) {
                oso.registerClass(User.class, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

                User user = new User("alice", true);
                assert OSO.allow(user, "foo", "bar");
                assert !OSO.allow("notauser", "foo", "bar");
            }

.. note::
    Type specializers automatically respect the
    **inheritance** hierarchy of our application classes. See our :doc:`/using/examples/inheritance` guide for an
    in-depth example of how this works.

Once a class is registered, its static methods can also be called from oso policies:

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor.name in User.superusers();

        .. code-block:: python
            :caption: app.py

            class User:
                @classmethod
                def superusers(cls):
                    """ Class method to return list of superusers. """
                    return ["alice", "bhavik", "clarice"]

            oso.register_class(User)

            user = User()
            user.name = "alice"
            assert(OSO.allow(user, "foo", "bar))

    .. group-tab:: Ruby

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor.name in User.superusers();

        .. code-block:: ruby
            :caption: app.rub

            class User
                # ...
                def self.superusers
                    ["alice", "bhavik", "clarice"]
                end
            end

            OSO.register_class(User)

        user = User.new("alice", true)
        raise "should be allowed" unless OSO.allow(user, "foo", "bar")

    .. group-tab:: Java

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor.name in User.superusers();

        .. code-block:: java
            :caption: User.java

            public static List<String> superusers() {
                return List.of("alice", "bhavik", "clarice");
            }

            public static void main(String[] args) {
                oso.registerClass(User.class, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

                User user = new User("alice", true);
                assert OSO.allow(user, "foo", "bar");
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

- **Application types** and their associated application data are available within policies.
- Types can be **registered** with oso, in order to:
    - Create instances of application types in policies
    - Leverage the inheritance structure of application types with **specialized rules**,
    supporting more sophisticated access control models.
- You can use **built-in methods** on primitive types & literals like strings and
  dictionaries, exactly as if they were application types.