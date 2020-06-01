=========================
Authorization Fundamentals
=========================

.. TODO (leina): high-level explanation here

.. _requests:

Requests
========

Authorization in oso always begins with a **request**.
A request might be an HTTP request from a client to an application,
or a query issued by an application backend to a data store.
Requests are made using the oso :doc:`/application-library/index`.

Authorization requests take the form:

    **Actor** A requests to take **Action** B on **Resource** C.

.. _actors:

Actors
------
Actors are the subjects of requests.   Actors
will often be application end-users, but could also represent
service users, api consumers, or other internal systems.

.. _resources:

Resources
---------
Resources are the objects of requests.
They represent components in the application to which access control is applied.

.. _actions:

Actions
-------
Actions are the verbs of requests.   They distinguish between different types of requests
for a given resource by indicating what the actor is attempting to do.


Policies
========

oso evaluates requests using authorization logic contained in **policies**.
Policies are written as code in a logic programming language called Polar.

Polar is designed to provide intuitive syntax for expressing authorization logic,
and seamlessly integrate application data structures into policies.
For more information on Polar, see the :ref:`Polar Language <polar>` documentation.

Policies are stored in Polar files (extension ``.polar``), which are loaded into the authorization engine using the
oso :doc:`/application-library/index`. Once loaded, policies can be used to evaulate requests.

Policies are made up of :ref:`rules <polar-rules>`.
Authorization requests are evaluated by :doc:`querying <language/polar-queries>` the engine
for a special kind of rule, the ``allow()`` rule.

Allow rules
===========

``allow()`` rules have the form::

   allow(actor, action, resource) := ...

When evaluating whether to allow a request, Polar will match the
request actor, action and resource with allow rules in the policy.

``allow()`` rules can be written over any of the types that polar supports
(see :ref:`basic-types` & :ref:`compound-types`).

Let's imagine we are using oso to write an authorization application for John F. Kennedy International Airport.

A simple Polar policy using strings might look like this::

   allow("alice", "board", "flight");
   allow("bob", "board", "flight");

The above policy says that both Alice and Bob are allowed to board a flight.

Note that the policy can be rewritten as::

   allow(actor, "board", "flight") :=
      actor = "alice" | "bob";

For more details on the above syntax, see :ref:`variables` and :ref:`disjunction`.

.. _inline-queries:

Inline queries
--------------

The behavior of a policy can be checked using **inline queries**, queries added directly
to a policy file. Inline queries are evaluated when the policy is loaded. The policy will fail
to load if any inline query does not succeed.

Inline queries use the following syntax::

  ?= allow("alice", "board", "flight");
  ?= allow("bob", "board", "flight");
  ?= !allow("charlie", "board", "flight");

All of the above queries should succeed, and the policy should load successfully.
The third query exhibits an important point: queries will fail unless specifically allowed
by a matching ``allow()`` rule. Put another way, we can think of Polar policies as "default-deny".

Going further
-------------

Our simple string-based policy has some obvious limitations.

We'd probably like to write rules that apply to all passengers, not just Alice and Bob.
Passengers shouldn't be able to board just *any* flight, but only flights for which they have boarding passes.
Maybe we'd like to check whether or not passengers have gone through security before allowing them to board.
And what about flight attendants? We might want to write separate rules for their boarding permissions.

To make the above work, our policy needs access to additional information that's likely stored in the airport's
internal system. oso solves this problem by letting us write policy rules over **application types**.

.. _application-types:

Application types
=================

Any type defined in our application can be registered with oso, and its attributes may be accessed from within
a Polar policy. Using types already defined in the application allows us to take advantage of the same domain
model we've already created when writing our app.

Application types are useful for defining policy objects, like actors and resources, that have
*attributes* we'd like to use in our policy logic.

Let's create classes to represent our passenger actors and flight resources.
For this example, we'll assume our application is written in Python, using oso's :doc:`Python application library <application-library/python>`.

.. code-block:: python

  from oso import polar_class

  @polar_class
  class Flight:
    def __init__(self, flight_number):
      ...

  @polar_class
  class Passenger:
    def __init__(self, name):
      ...

    def boarding_pass(self):
      return db.get_boarding_passes(passenger=self)

In this example, we assume that ``Flight`` has a ``flight_number`` property that
can be used in the policy.  ``Passenger`` has a ``boarding_pass`` method that will look
up boarding passes in the database. Notice that even though we never registered the ``BoardingPass``
type returned from ``boarding_pass``, Polar can still understand it.

.. TODO (dhatch): Add arguments to method

Now that we have registered our types with Polar, we can use the following
policy to check passengers' boarding passes:

.. code-block:: polar

  allow(actor: Passenger, "board", resource: Flight) :=
      actor = Passenger{},
      resource = Flight{},
      actor.boarding_pass.flight_number = resource.flight_number;

``actor`` and ``resource`` are now :ref:`variables <variables>`, and will be bound to
whatever objects are passed into the request.
The ``param: Type`` syntax in the rule head is a :ref:`type specializer <inheritance>`. It
indicates that that the rule will only be evaluated if the actor is an instance of ``Passenger``
and the resource is an instance of ``Flight``.

The ``.`` operator retrieves attributes on the objects from within the application.
Notice that for methods with no arguments, the ``()`` can be elided.

Let's try making an authorization request with our new policy. In Python,
requests are made using the :py:meth:`oso.allow` method::

  def make_request:
    # create shared state and load the policy
    oso = get_oso()

    passenger = Passenger(name="Alice")
    flight = Flight(flight_number=123)

    if oso.allow(passenger, "board", flight):
      print("Alice can board!")


.. _inheritance:

Specialized rules & inheritance
===============================

Polar natively understands your application's types.  We can use this knowledge to
write rules that only apply to certain types of resources or actors. These rules are called
*specialized rules*.

Let's suppose we want to control flight attendant's access to flights.  We have a class in our
application called ``FlightAttendant`` that represents this actor.  We write a new rule:

.. code-block:: polar

  allow(flight_attendant: FlightAttendant, "board", flight: Flight) :=
    flight_attendant.airline = flight.airline;

Notice the new syntax we have used in the rule head: ``param: Type``. This form indicates that
that the rule will only be evaluated if the parameter has the type specified by ``Type``.  Polar will only
evaluate this allow rule if the ``actor`` is a ``FlightAttendant``, and the resource is a ``Flight``.

Flight attendants are not the only type of employee that needs to board the flight.  We also need to let
pilots aboard, with the same logic.  We have a class in our application called ``AirlineEmployee`` that is a
superclass of both ``FlightAttendant`` and ``Pilot``.  Polar understands our application type hiearchy.  We can
write a rule:

.. code-block:: polar

  allow(employee: AirlineEmployee, "board", flight: Flight) :=
    employee.airline = flight.airline;

This rule matches both ``Pilot`` and ``FlightAttendant`` since they are both ``AirlineEmployee`` subclasses.

.. TODO (dhatch): This would be a great spot to intro groups.


Summary
=======
- In oso, authorization begins with a **request**, which is evaluated against a Polar **policy**.
- Policies are made up of **rules**, and ``allow()`` rules are used to grant permissions.
- **Application
  types** can be exposed to Polar in order to make application data available from within policies.
- The inheritance structure of application types can be leveraged in the policy with **specialized rules**,
  supporting more sophistiscated access control models.

You've got the fundamentals down!
To see more of oso in action, check out our :doc:`authorization model guides </auth-models/index>`.