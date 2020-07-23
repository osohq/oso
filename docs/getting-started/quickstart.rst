==========
Quickstart
==========

.. todo::
    Speed up the getting started so we can do time to dopamine in 5 minutes.

If you don't know what oso is, head back to :doc:`the introduction </index>`. If you've
never used oso before and want to get up-and-running quickly, this guide is for
you.

In general, it takes less than 5 minutes to add oso to an existing application
and begin writing an authorization policy. In the next 15 minutes, we're going
to create a simple web server with no authorization, add oso to the project,
and then write our first policy. We encourage you to code along in your local
environment!

A basic web server
==================

Our application serves data about expenses submitted by users.

We have a simple ``Expense`` class, and some stored data in the ``EXPENSES`` dictionary.
Our web server contains some simple logic to filter out bad requests and not much else.

.. tabs::

  .. group-tab:: Python

    .. literalinclude:: /examples/getting-started/python/server.py
      :class: copybutton
      :caption: :fab:`python` server.py :download:`(link) </examples/getting-started/python/server.py>`
      :language: python

  .. group-tab:: Ruby

    .. literalinclude:: /examples/getting-started/ruby/server.rb
      :class: copybutton
      :caption: :fas:`gem` server.rb :download:`(link) </examples/getting-started/ruby/server.rb>`
      :language: ruby

  .. group-tab:: Java

    .. literalinclude:: /examples/getting-started/java/server/Server.java
      :class: copybutton
      :caption: :fab:`java` Server.java :download:`(link) </examples/getting-started/java/server/Server.java>`
      :language: java

    .. literalinclude:: /examples/getting-started/java/server/Expense.java
      :class: copybutton
      :caption: :fab:`java` Expense.java :download:`(link) </examples/getting-started/java/server/Expense.java>`
      :language: java

If the request path matches the form ``/expenses/:id`` and ``:id`` is the ID of
an existing expense, we respond with the expense data. Otherwise, we return
``"Not Found!"``.

Let's use `cURL <https://curl.haxx.se/>`_ to check that everything's working.
We'll first start our server...

.. tabs::
  .. group-tab:: Python

    .. code-block:: console

      $ python server.py
      running on port 5050

  .. group-tab:: Ruby

    .. code-block:: console

      $ ruby server.rb
      [2020-07-15 00:35:52] INFO  WEBrick 1.3.1
      [2020-07-15 00:35:52] INFO  ruby 2.4.10 (2020-03-31) [x86_64-linux]
      [2020-07-15 00:35:52] INFO  WEBrick::HTTPServer#start: pid=537647 port=5050

  .. group-tab:: Java

    .. code-block:: console

        $ javac Server.java
        $ java Server
        Server running on /127.0.0.1:5050

...and then, in another terminal, make some requests to our running server:

.. code-block:: console

  $ curl localhost:5050/expenses/1
  Expense(amount=500, description='coffee', submitted_by='alice@example.com')
  $ curl localhost:5050/expenses/4
  Not Found!

Our server currently has no authorization --- anyone can view any expense. The
bigwigs at Example.com, Inc. are none too pleased with the lax security, so
let's create an access policy with oso!

Adding oso
==========

.. admonition:: Installation

  In order to write our first authorization policy, we first need to add oso to
  our application. If you don't already have it :doc:`installed </getting-started/download/index>`, go ahead and
  do so now:

  .. todo::
    replace the hard-coded version number(s) in the below snippet with the current latest version on RubyGems... somehow.

  .. tabs::
    .. group-tab:: Python

      .. code-block:: console

        $ pip install oso

    .. group-tab:: Ruby

      .. code-block:: console

        $ gem install oso-oso

    .. group-tab:: Java

      .. todo:: Java install

      Download :download:`oso-0.2.5.jar </examples/getting-started/java/lib/oso-0.2.5.jar>`.

      Then build and run your server with:

      .. code-block:: console

        $ javac -cp oso-0.2.5.jar:. Server.java
        $ java -cp oso-0.2.5.jar:. Server



Now that we've installed oso, we can import it into our project and construct
a new ``Oso`` instance that will serve as our authorization engine.

Here's an updated version of our web server code from before with the new lines
highlighted:

.. tabs::
  .. group-tab:: Python

    .. literalinclude:: /examples/getting-started/python/server-with-oso.py
      :caption: :fab:`python` server.py :download:`(link) </examples/getting-started/python/server-with-oso.py>`
      :language: python
      :class: copybutton
      :emphasize-lines: 3,5-6,31-32,39-42

    And a new empty Polar policy file:

    .. literalinclude:: /examples/getting-started/polar/expenses-01.polar
      :caption: :fa:`oso` expenses.polar

  .. group-tab:: Ruby

    .. literalinclude:: /examples/getting-started/ruby/server-with-oso.rb
      :caption: :fas:`gem` server.rb :download:`(link) </examples/getting-started/ruby/server-with-oso.rb>`
      :language: ruby
      :class: copybutton
      :emphasize-lines: 1,4-5,25-26,32-36

    And a new empty Polar policy file:

    .. literalinclude:: /examples/getting-started/polar/expenses-01.polar
      :caption: :fa:`oso` expenses.polar

  .. group-tab:: Java

    .. literalinclude:: /examples/getting-started/java/server-with-oso/Server.java
      :caption: :fab:`java` Server.java :download:`(link) </examples/getting-started/java/server-with-oso/Server.java>`
      :language: java
      :class: copybutton
      :emphasize-lines: 4,10-15,37-41

    And a new empty Polar policy file:

    .. literalinclude:: /examples/getting-started/polar/expenses-01.polar
      :caption: :fa:`oso` expenses.polar


And just like that, we're ready to start asking our global ``Oso`` instance to
make authorization decisions!

Decisions, decisions...
=======================

The ``Oso`` instance exposes an ``allow()`` method that takes three
arguments, **actor**, **action**, and **resource**:


.. tabs::
  .. group-tab:: Python

    .. literalinclude:: /examples/getting-started/python/allow-01.py
      :language: python
      :lines: 11-13

  .. group-tab:: Ruby

      .. literalinclude:: /examples/getting-started/ruby/allow-01.rb
        :language: ruby
        :lines: 4-6

  .. group-tab:: Java

    .. literalinclude:: /examples/getting-started/java/allow-01.java
      :language: java
      :lines: 5-8
      :dedent: 8

The above method call returns ``true`` if the **actor** ``"alice@example.com"`` may
perform the **action** ``"GET"`` on the
**resource** ``EXPENSES[1]``. We're using ``"GET"`` here to match up with the HTTP
verb used in our server, but this could be anything.

.. note:: For more on **actors**, **actions**, and **resources**, check out
  :doc:`/using/key-concepts`.

oso's authorization system is deny-by-default. Since we haven't yet written any
policy code, Alice is not allowed to view expenses. To see that in action,
start a REPL session and follow along:

.. tabs::
  .. group-tab:: Python

    Run: ``python``

    .. code-block:: pycon


      >>> from server import *
      >>> oso
      <oso.Oso object at 0x7f267494dc70>
      >>> alice = "alice@example.com"
      >>> expense = EXPENSES[1]
      >>> oso.allow(alice, "GET", expense)
      False

    We can add a rule explicitly allowing Alice to GET expenses...

    .. literalinclude:: /examples/getting-started/polar/expenses-02.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    ...which we can load into our oso instance:

    .. code-block:: pycon

      >>> oso.load_file("expenses.polar")

    ...and now Alice has the power...

    .. code-block:: pycon

      >>> oso.allow(alice, "GET", expense)
      True

    ...and everyone else is still denied:

    .. code-block:: pycon

      >>> OSO.allow("bhavik", "GET", expense)
      False


  .. group-tab:: Ruby

    Run: ``irb``

    .. code-block:: irb

        irb(main):001:0> require "./server"
        => true
        irb(main):002:0> alice = "alice@example.com"
        => "alice@example.com"
        irb(main):003:0> expense = EXPENSES[1]
        => #<Expense:0x00564efc19e640 @amount=500, @description="coffee", @submitted_by="alice@example.com">
        irb(main):004:0> OSO.allow(actor: alice, action: "GET", resource: expense)
        => false

    We can add a rule explicitly allowing Alice to view expenses...

    .. literalinclude:: /examples/getting-started/polar/expenses-02.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    ...which we can load into our oso instance:

    .. code-block:: irb

      irb(main):005:0> OSO.load_file("expenses.polar")
      => #<Set: {"expenses.polar"}>

    ...and now Alice has the power...

    .. code-block:: irb

      irb(main):005:0> OSO.allow(actor: "alice", action: "GET", resource: "expense")
      => true

    ...and everyone else is still denied:

    .. code-block:: irb

      irb(main):006:0> OSO.allow(actor: "bhavik", action: "GET", resource: "expense")
      => false

  .. group-tab:: Java

    Run: ``jshell --class-path oso-0.2.5.jar Server.java``

    .. code-block:: java

        jshell> Oso oso = new Oso()

        jshell> String alice = "alice@example.com"
        alice ==> "alice@example.com"

        jshell> Expense expense = Server.EXPENSES[1]
        expense ==> Expense(5000, software, alice@example.com)

        jshell> oso.allow(alice, "GET", expense)
        $12 ==> false

    We can add a rule explicitly allowing Alice to view expenses...

    .. literalinclude:: /examples/getting-started/polar/expenses-02.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    ...which we can load into our oso instance:

    .. code-block:: java

      jshell> oso.loadFile("expenses.polar")

    ...and now Alice has the power...

    .. code-block:: java

      jshell> oso.allow(alice, "GET", expense)
      $14 ==> true

    ...and everyone else is still denied:

    .. code-block:: java

      jshell> oso.allow("bhavik", "GET", expense)
      $15 ==> false

.. note::
  Each time you load a file, it will load the policy
  **without** clearing what is already loaded in. Be sure to
  clear oso using ``Oso.clear`` or create a new instance if you want
  to try adding a few new rules.

When we ask oso for a policy decision via ``allow``, the oso engine
searches through its knowledge base to determine whether the provided
**actor**, **action**, and **resource** satisfy any **allow** rules.

In the above case, we passed in ``alice`` as the **actor**, ``"GET"`` as the
**action**, and ``EXPENSE[1]`` as the **resource**, satisfying the
``allow("alice@example.com", "GET", _expense);`` rule.
When we pass in ``"bhavik@example.com"`` as
the actor, the rule no longer succeeds because the string ``"bhavik@example.com"`` does not
match the string ``"alice@example.com"``.

.. note:: For a deeper introduction to writing authorization rules with oso,
  see :doc:`/using/key-concepts`.

Authorizing HTTP requests
=========================

In lieu of setting up real identity and authentication systems,
in the example we used a custom HTTP header to indicate that
a request is "authenticated" as a
particular user. The header value will be an email address, e.g.,
``"alice@example.com"``. We'll pass it to ``allow`` as the **actor**
and we'll use the HTTP method as the **action**.

Finally, the **resource** is the expense retrieved from our stored expenses.

Assuming you added the rule from the previous step:

.. literalinclude:: /examples/getting-started/polar/expenses-02.polar
  :caption: :fa:`oso` expenses.polar
  :class: copybutton

We can test everything works by
starting the new server and making a valid request:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(amount=500, description='coffee', submitted_by='alice@example.com')
  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Not Authorized!

Rules over dynamic data
-----------------------

It's nice that Alice can now view expenses, but it would be really onerous if
we had to write a separate rule for every single actor we wanted to authorize.
Luckily, we don't!

Let's replace our static rule checking that the provided email matches
``"alice@example.com"`` with a dynamic one that checks that the provided email
ends in ``"@example.com"``. That way, everyone at Example.com, Inc. will be
able to view expenses, but no one outside the company will be able to:

.. tabs::
  .. group-tab:: Python

    .. literalinclude:: /examples/getting-started/polar/expenses-03-py.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    .. |str_endswith| replace:: the ``str.endswith`` method
    .. _str_endswith: https://docs.python.org/3/library/stdtypes.html#str.endswith

    We bind the provided email to the ``actor`` variable in the rule head and then
    perform the ``.endswith("@example.com")`` check in the rule body. If you
    noticed that the ``.endswith`` call looks pretty familiar, you're right on ---
    oso is actually calling out to |str_endswith|_ defined in the Python standard
    library. The **actor** value passed to oso is a Python string, and oso allows us
    to call any ``str`` method from Python's standard library on it.

  .. group-tab:: Ruby

    .. literalinclude:: /examples/getting-started/polar/expenses-03-rb.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    .. |string_end_with| replace:: the ``String#end_with?`` method
    .. _string_end_with: https://ruby-doc.org/core/String.html#method-i-end_with-3F

    We bind the provided email to the ``actor`` variable in the rule head and then
    perform the ``.end_with?("@example.com")`` check in the rule body. If you
    noticed that the ``.end_with?`` call looks pretty familiar, you're right on ---
    oso is actually calling out to |string_end_with|_ defined in the Ruby standard
    library. The **actor** value passed to oso is a Ruby string, and oso allows us
    to call any ``String`` method from Ruby's standard library on it.

  .. group-tab:: Java

    .. literalinclude:: /examples/getting-started/polar/expenses-03-java.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    .. |string_endsWith| replace:: the ``String#endsWith?`` method
    .. _string_endsWith: https://www.w3schools.com/java/ref_string_endswith.asp

    We bind the provided email to the ``actor`` variable in the rule head and then
    perform the ``.endsWith("@example.com")`` check in the rule body. If you
    noticed that the ``.endsWith`` call looks pretty familiar, you're right on ---
    oso is actually calling out to |string_endsWith|_ defined in the Java standard
    library. The **actor** value passed to oso is a Ruby string, and oso allows us
    to call any ``String`` method from Java's standard library on it.

Once we've added our new dynamic rule and restarted the web server, every user
with an ``@example.com`` email should be allowed to view any expense:

.. code-block:: console

  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Expense(...)

If a user's email doesn't end in ``"@example.com"``, the rule fails, and they
are denied access:

.. code-block:: console

  $ curl -H "user: bhavik@example.org" localhost:5050/expenses/1
  Not Authorized!

And that's just the tip of the iceberg. You can register *any* application object with
oso and then leverage it in your application's authorization policy.
In the next section, we'll update
our existing policy to leverage the ``Expense`` class defined in our
application.

Writing authorization policy over application data
==================================================

At this point, the higher-ups at Example.com, Inc. are still not satisfied with
our access policy that allows all employees to see each other's expenses. They
would like us to modify the policy such that employees can only see their own
expenses.

To accomplish that, we can replace our existing rule with:

.. tabs::

  .. group-tab:: Python

    .. literalinclude:: /examples/getting-started/polar/expenses-04.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

  .. group-tab:: Ruby

    .. literalinclude:: /examples/getting-started/polar/expenses-04.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

  .. group-tab:: Java

    .. literalinclude:: /examples/getting-started/polar/expenses-04-java.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

Behind the scenes, oso looks up the ``submitted_by`` field on the provided
``Expense`` instance and compares that value against the provided **actor**.
And just like that, an actor can only see an expense if they submitted the expense.

Now Alice can see her own expenses but not Bhavik's:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(...)
  $ curl -H "user: alice@example.com" localhost:5050/expenses/3
  Not Authorized!

And vice-versa:

.. code-block:: console

  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Not Authorized!
  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/3
  Expense(...)

We encourage you to play around with the current policy and experiment with
adding your own rules!

For
example, if you have ``Expense`` and ``User`` classes defined in your
application, you could write a policy rule in oso that says a ``User`` may
approve an ``Expense`` if they manage the ``User`` who submitted the expense
and the expense's amount is less than $100.00:


.. code-block:: polar
  :class: no-select

  allow(approver, "approve", expense) if
      approver = expense.submitted_by.manager
      and expense.amount < 10000;

In the process of evaluating that rule, the oso engine would call back into the
application in order to make determinations that rely on application data, such
as:

- Which user submitted the expense in question?
- Who is their manager?
- Is their manager the approver?
- Does the expense's ``amount`` field contain a value less than $100.00?

.. note:: For more on leveraging application data in an oso policy, check out
  :doc:`/using/policies/application-types`.



Summary
=======

We just blitzed through a ton of stuff:

* Installing oso.
* Setting up our app to enforce the policy decisions made by oso.
* Writing authorization rules over static and dynamic application data.

If you're interested in what sets oso apart from existing authorization
solutions, check out :doc:`/getting-started/overview`. If you want to learn more about
authorization in oso, including common patterns like :doc:`/using/examples/rbac`
and :doc:`/using/examples/abac`, we recommend continuing on to the
:doc:`/using/key-concepts` guide. For more details on the logic programming
language we used to write our authorization policies, head on over to the
:doc:`/understand/language/polar-fundamentals` guide.
