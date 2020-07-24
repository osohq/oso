
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

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor matches User{name: "alice"};

        .. code-block:: python
            :caption: app.py

            oso.register_class(User)

            user = User()
            user.name = "alice"
            assert(OSO.allow(user, "foo", "bar))
            assert(not OSO.allow("notauser", "foo", "bar"))

    .. group-tab:: Ruby

        .. code-block:: polar
            :caption: policy.polar

            allow(actor: User, action, resource) if actor matches User{name: "alice", is_admin: true};

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

        .. code-block:: java
            :caption: User.java

            public static void main(String[] args) {
                oso.registerClass(User, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

                User user = new User("alice", true);
                assert OSO.allow(user, "foo", "bar");
                assert !OSO.allow("notauser", "foo", "bar");
            }

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
                oso.registerClass(User, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");

                User user = new User("alice", true);
                assert OSO.allow(user, "foo", "bar");
            }

