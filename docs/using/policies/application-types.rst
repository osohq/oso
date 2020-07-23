.. _application-types:

=================
Application types
=================

Any type defined in our application can be registered with oso, and its
attributes may be accessed from within a policy. Using application types
lets us take advantage of our app's existing domain model.

Let's continue our :ref:`airport authorization example <airport>` from
the :doc:`/using/key-concepts` document. Suppose we have some simple Python
classes that represent airport passengers and flight resources.

.. code-block:: python

  from oso import polar_class

  @polar_class
  class Flight:
    def __init__(self, flight_number):
      self.flight_number = flight_number

  @polar_class
  class Passenger:
    def __init__(self, name):
      self.name = name

    def boarding_pass(self):
      return db.get_boarding_passes(passenger=self.name)

The ``@polar_class`` decorator **registers** an application class with oso
so that it can be recognized as a type in our policy. Here's one way we might
use those types::

  allow(actor: Passenger, "board", resource: Flight) if
      actor.boarding_pass.flight_number = resource.flight_number;

This rule says roughly: "allow any passenger to board a flight if
the flight number matches the one on their boarding pass". In this rule,
the ``actor`` and ``resource`` parameters are :ref:`variables <variables>`
with :ref:`type specializers <inheritance>`: if the actor supplied in
an authorization query is an instance of the ``Passenger`` class, then
the ``actor`` variable will be bound to it; likewise for ``resource``
with the ``Flight`` class. These specializations are necessary because
the body of the rule accesses attributes such as ``boarding_pass`` and
``flight_number`` that may not exist on other types of objects.

Let's look at the body now. The body consists of a single ``=`` expression,
which we can think of as a simple equality check. The left-hand side
is a chained lookup using the ``.`` operator: it first looks up the
``boarding_pass`` method on the ``actor`` (a ``Passenger`` instance),
calls the method, and then looks up the ``flight_number`` attribute on
the returned object (whatever that is). The right-hand side looks up the
``flight_number`` attribute of the ``resource`` (a ``Flight`` instance).
The ``=`` expression as a whole succeeds just when the two flight numbers
are equal.

Let's try making an authorization query against our new policy. In Python,
we can use the :py:meth:`oso.allow` method::

  def authorize():
    # create shared state and load the policy
    oso = get_oso()
    ...
    if oso.allow(Passenger(name="Alice"), "board", Flight(flight_number=123)):
      print("Alice can board!")

Since the ``actor`` and ``resource`` arguments are instances of the expected
types, the authorization query should succeed just when ``db.get_boarding_passes``
returns a boarding pass with flight number ``123``.

.. _inheritance:

Inheritance
-----------

The type specializations that we used above automatically respect the
inheritance hierarchy of our application classes. Let's suppose we want
to control flight attendant's access to flights. We could have a class
in our application called ``FlightAttendant`` that represents this type
of actor.  We write a new rule:

.. code-block:: polar

  allow(flight_attendant: FlightAttendant, "board", flight: Flight) if
    flight_attendant.airline = flight.airline;

Notice the new syntax we have used in the rule head: ``param: Type``.
This form indicates that that the rule will only be evaluated if the parameter
has the type specified by ``Type``.  This allow rule evaluates if the ``actor``
is a ``FlightAttendant``, and the resource is a ``Flight``.

Flight attendants are not the only type of employee that needs to board the
flight.  We also need to let pilots aboard, with the same logic.  We have a
class in our application called ``AirlineEmployee`` that is a superclass of both
``FlightAttendant`` and ``Pilot``.  oso understands our application type
hierarchy.  We can write a rule:

.. code-block:: polar

  allow(employee: AirlineEmployee, "board", flight: Flight) if
    employee.airline = flight.airline;

This rule matches both a ``Pilot`` and ``FlightAttendant`` since they are both
subclasses of ``AirlineEmployee``.

.. todo::
   This would be a great spot to intro groups.

.. _built-in-types:

Built-in types
--------------

Methods called on Polar built-ins (``str``, ``dict``, ``number`` & ``list``)
call methods on the corresponding language type. That way you can use
familiar methods like ``str.startswith()`` on strings regardless of whether
they originated in your application or as a literal in your policy.
This applies to all of the Polar :ref:`primitive types <basic-types>`:
strings, lists, dictionaries, and numbers, in any supported application
language.

.. warning:: Do not attempt to mutate a literal using a method on it.
  Literals in Polar are constant, and any changes made to such objects
  by calling a method will not be persisted.

.. todo:: more info on this, link to each language guide

Summary
=======

- **Application types** can be registered with oso to make application data
  available within policies.
- The inheritance structure of application types can be leveraged in the policy
  with **specialized rules**, supporting more sophisticated access control
  models.
- You can use built-in methods on primitive types & literals like strings and
  dictionaries, exactly as if they were application types.


.. JAVA EXAMPLES

.. For example:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor, action, resource) if actor.isAdmin;
..
.. The above rule expects the ``actor`` variable to be a Java instance with the field ``isAdmin``.
.. The Java instance is passed into oso with a call to ``Oso.allow``:
..
.. .. TODO: add link to javadocs
..
.. .. code-block:: java
..    :caption: User.java
..
..    public class User {
..       public boolean isAdmin;
..       public String name;
..
..       public User(String name, boolean isAdmin) {
..          this.isAdmin = isAdmin;
..          this.name = name;
..       }
..
..       public static void main(String[] args) {
..          User user = new User("alice", true);
..          assert oso.allow(user, "foo", "bar");
..       }
..    }
..
..
.. The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has a field
.. called ``isAdmin``, it is evaluated by the Polar rule and found to be true.

.. Java instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Java class has been **registered** using
.. the ``registerClass()`` method.
..
.. Registering classes also makes it possible to use :ref:`specialization` and the
.. :ref:`operator-matches` with the registered class:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor: User, action, resource) if actor matches User{name: "alice", isAdmin: true};
..
.. .. code-block:: java
..    :caption: User.java
..
..       public static void main(String[] args) {
..          oso.registerClass(User, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");
..
..          User user = new User("alice", true);
..          assert oso.allow(user, "foo", "bar");
..          assert !oso.allow("notauser", "foo", "bar");
..       }
..
.. Once a class is registered, its static methods can also be called from oso policies:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor: User, action, resource) if actor.name in User.superusers();
..
.. .. code-block:: java
..    :caption: User.java
..
..       public static List<String> superusers() {
..          return List.of("alice", "bhavik", "clarice");
..       }
..
..       public static void main(String[] args) {
..          oso.registerClass(User, (args) -> new User((String) args.get("name"), (boolean) args.get("isAdmin")), "User");
..
..          User user = new User("alice", true);
..          assert oso.allow(user, "foo", "bar");
..       }

.. RUBY EXAMPLES

.. For example:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor, action, resource) if actor.is_admin?;
..
.. The above rule expects the ``actor`` variable to be a Ruby instance with the attribute ``is_admin?``.
.. The Ruby instance is passed into oso with a call to ``allow()``:
..
.. .. code-block:: ruby
..    :caption: app.rb
..
..    class User
..       attr_reader :name
..       attr_reader :is_admin
..
..       def initialize(name, is_admin)
..          @name = name
..          @is_admin = is_admin
..       end
..    end
..
..    user = User.new("alice", true)
..    raise "should be allowed" unless oso.allow(user, "foo", "bar")
..
.. The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
.. called ``is_admin``, it is evaluated by the policy and found to be true.

.. Registering classes also makes it possible to use :ref:`specialization` and the
.. :ref:`operator-matches` with the registered class:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor: User, action, resource) if actor matches User{name: "alice", is_admin: true};
..
.. .. code-block:: ruby
..    :caption: app.rb
..
..    OSO.register_class(User)
..    user = User.new("alice", true)
..    raise "should be allowed" unless oso.allow(user, "foo", "bar")
..    raise "should not be allowed" unless not oso.allow(user, "foo", "bar")
..
.. Once a class is registered, its class methods can also be called from oso policies:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor: User, action, resource) if actor.name in User.superusers();
..
.. .. code-block:: ruby
..    :caption: app.rub
..
..    class User
..       # ...
..       def self.superusers
..          ["alice", "bhavik", "clarice"]
..       end
..    end
..
..    oso.register_class(User)
..
..    user = User.new("alice", true)
..    raise "should be allowed" unless oso.allow(user, "foo", "bar")

.. PYTHON EXAMPLES

.. For example:
..
.. .. code-block:: polar
..    :caption: policy.polar
..
..    allow(actor, action, resource) if actor.is_admin;
..
.. The above rule expects the ``actor`` variable to be a Python instance with the attribute ``is_admin``.
.. The Python instance is passed into oso with a call to :py:meth:`~oso.Oso.allow`:
..
.. .. code-block:: python
..    :caption: app.py
..
..    user = User()
..    user.is_admin = True
..    assert(oso.allow(user, "foo", "bar))
..
.. The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
.. called ``is_admin``, it is evaluated by the policy and found to be true.

.. Registering classes also makes it possible to use :ref:`specialization` and the
.. :ref:`operator-matches` with the registered class:

.. .. code-block:: polar
..    :caption: policy.polar

..    allow(actor: User, action, resource) if actor matches User{name: "alice"};

.. .. code-block:: python
..    :caption: app.py

..    oso.register_class(User)

..    user = User()
..    user.name = "alice"
..    assert(oso.allow(user, "foo", "bar))
..    assert(not oso.allow("notauser", "foo", "bar"))

.. Once a class is registered, its class methods can also be called from oso policies:

.. .. code-block:: polar
..    :caption: policy.polar

..    allow(actor: User, action, resource) if actor.name in User.superusers();

.. .. code-block:: python
..    :caption: app.py

..    class User:
..       @classmethod
..       def superusers(cls):
..          """ Class method to return list of superusers. """
..          return ["alice", "bhavik", "clarice"]

..    oso.register_class(User)

..    user = User()
..    user.name = "alice"
..    assert(oso.allow(user, "foo", "bar))