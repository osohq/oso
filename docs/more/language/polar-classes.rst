.. _polar-classes:

.. todo:: This guide is out of date and not included anywhere.

===========================
Classes & specialized rules
===========================

.. note::
   This guide assumes you have familiarity with the
   :doc:`polar-syntax` and :doc:`polar-foundations`.

.. note::
   This is an area of active development! Syntax is likely to change
   and evolve.

Now that we've seen how to write basic rules and data structures,
let's explore the Polar class and rule specialization system.

.. fixme: Make sure the following statement is true.

.. highlight:: polar

Associated with every data type in Polar is a *class*.
You can define your own classes, too, to represent data that is
relevant to your application. For example, if you want to restrict
access to a particular type of object in your domain—widgets, say—you
could model them with a class::

  class(Widget);

This defines a new data type named ``Widget``. If we want to express
a hierarchy of types, we can specify a superclass (or a tuple of superclasses) from which to inherit structure and behavior::

  class(DooDad, Widget);

This says that every ``DooDad`` is a ``Widget``. Classes are
instantiated using the tagged dictionary syntax::

  Widget{id: 1, owner: User{first: "sam", last: "scott"}}
  DooDad{id: 101, spinner: "super"}

Here's a sample user hierarchy::

  class(User);
  class(Admin, User);
  class(Supervisor, Admin);
  class(Guest, User);
  class(Anonymous, Guest);
  class(Invited, Guest);
  class(Sudoer, (Admin, Invited));

And here are some actions such users might perform on widgets::

  class(Action);
  class(Read, Action);
  class(Adjust, Action);
  class(Frob, Adjust);
  class(Twiddle, Adjust);
  class(Tweak, Adjust);
  class(Smash, Action);

Having defined these, we can now write rules with parameters that
*specialize* on instances of these classes; i.e., that will only
be *applicable* (match) if the argument supplied at query time is
of the specified class (or any subclass). We write this by adding
a type annotation to a predicate parameter in the head of a rule
using the syntax ``param: Class``. For example::

  allow(user: User, action: Read, resource: Widget);

This says that any instance of ``User`` (or any subclass) may
``Read`` (or any sub-action, though there aren't any here) any
``Widget`` (or any subclass, e.g., ``DooDad``). Here are some
other rules we could write::

  allow(user: Admin, action: Adjust, resource: Widget);
  allow(user: Supervisor, action: Smash, resource: Widget);
  allow(user: Guest, action: Tweak, resource: Widget);
  allow(user: Invited, action: Adjust, resource: DooDad);

Further restrictions, e.g., on the values of certain fields,
may be added in the bodies::

  allow(user: User, action: Smash, resource: Widget) if
    user.hammer.name = "Mjölnir";

Ordinary parameters matched via unification may be mixed with
class-specialized parameters::

  allow(person("sam", "scott"), action, resource: Widget);

Rules are run in *most-specific-first* order, where specificity
is determined by the class hierarchy of each argument, considered
in left-to-right order. This is important if a rule includes a
``cut`` operator; e.g.::

  allow(user: Invited, action: Frob, resource: DooDad) if
    cut();

This says that an ``Invited`` user may ``Frob`` a ``DooDad``,
and no less specific rule (i.e., one whose first parameter is
specialized on ``Guest`` or ``User``) will be considered. Normally
all applicable rules are tried, exactly as with rules matched
by unification only.

You can also write rules specialized on :doc:`application classes <../application-library/externals>`,
i.e., classes written in your application programming language.
These work almost exactly the same way as native Polar classes,
except that the specifics of the rule ordering may depend on
that language's native class system.
