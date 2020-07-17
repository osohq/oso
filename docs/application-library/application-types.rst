.. _application-types:
=================
Application types
=================

Any type defined in our application can be registered with Polar, and its
attributes may be accessed from within a policy. Using application types
lets us take advantage of our app's existing domain model.

Let's continue our :ref:`airport authorization example <airport>` from
the :doc:`/auth-fundamentals` document. Suppose we have some simple Python
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

The ``@polar_class`` decorator **registers** an application class with Polar
so that it can be recognized as a type. Here's one way we might use those types::

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
has the type specified by ``Type``.  Polar will only evaluate this allow rule
if the ``actor`` is a ``FlightAttendant``, and the resource is a ``Flight``.

Flight attendants are not the only type of employee that needs to board the flight.
We also need to let pilots aboard, with the same logic.  We have a class in our
application called ``AirlineEmployee`` that is a superclass of both ``FlightAttendant``
and ``Pilot``.  Polar understands our application type hiearchy.  We can
write a rule:

.. code-block:: polar

  allow(employee: AirlineEmployee, "board", flight: Flight) if
    employee.airline = flight.airline;

This rule matches both a ``Pilot`` and ``FlightAttendant`` since they are both
subclasses of ``AirlineEmployee``.

.. TODO (dhatch): This would be a great spot to intro groups.

.. _built-in-types:

Built-in types
--------------

Because your application objects probably use your language's built-in
primitive types such as ``str``, ``dict``, and ``int``, Polar allows you
to use methods on those types for its built-ins, too. That way you can use
familiar methods like ``str.startswith()`` on strings regardless of whether
they originated in your application or as a literal in your policy.
This applies to all of the Polar :ref:`primitive types <basic-types>`:
strings, lists, dictionaries, and numbers, in any supported application
language.

.. warning:: Do not attempt to mutate a literal using a method on it.
  Literals in Polar are constant, and any changes made to such objects
  on the application side will not be reflected back to Polar.

Summary
=======
- **Application types** can be registered with Polar to make application data available within policies.
- The inheritance structure of application types can be leveraged in the policy with **specialized rules**,
  supporting more sophistiscated access control models.
- You can use built-in methods on primitive types like strings and
  dictionaries, exactly as if they were application types.
