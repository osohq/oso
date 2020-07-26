==========
Quickstart
==========

oso helps developers build authorization into their applications. If you've
never used oso before and want to get up-and-running quickly, this guide is for
you.

In general, it takes less than 5 minutes to add oso to an existing application
and begin writing an authorization policy.

In the next 15 minutes, we're going to add oso to our project, write our first policy,
create a simple web server with no authorization, and write some rules for it.
We encourage you to code along in your local environment!

Expenses Application
====================

Our application serves data about expenses submitted by users.

To start with, we have a simple ``Expense`` class, and some stored data in the
``EXPENSES`` dictionary.

.. tabs::

  .. group-tab:: Python

    .. literalinclude:: /examples/quickstart/python/expense.py
      :class: copybutton
      :caption: :fab:`python` expense.py :download:`(link) </examples/quickstart/python/expense.py>`
      :language: python

  .. group-tab:: Ruby

    .. literalinclude:: /examples/quickstart/ruby/expense.rb
      :class: copybutton
      :caption: :fas:`gem` expense.rb :download:`(link) </examples/quickstart/ruby/expense.rb>`
      :language: ruby

  .. group-tab:: Java

    .. literalinclude:: /examples/quickstart/java/Expense.java
      :class: copybutton
      :caption: :fab:`java` expense.java :download:`(link) </examples/quickstart/java/Expense.java>`
      :language: java

We'll need our application to be able to control who has access to this data.
Before we add a web server and start making some requests, lets see if we can get
some basic authorization in place!

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

      oso supports Python versions **>= 3.6**

      .. code-block:: console

        $ pip install oso

    .. group-tab:: Ruby

      oso supports Ruby versions **>= 2.4**

      .. code-block:: console

        $ gem install oso-oso

    .. group-tab:: Java

      oso supports Java versions **>= 9**

      Download :download:`oso-0.2.5.jar </examples/quickstart/java/lib/oso-0.2.5.jar>`.

      Either add this to your Java project libraries,
      load using your IDE,
      or build from the command line with:

      .. code-block:: console

        $ javac -cp oso-0.2.5.jar:. Expense.java


Now that we've installed oso, we can import it into our project and construct
a new ``Oso`` instance that will serve as our authorization engine.

Decisions, decisions...
=======================

The ``Oso`` instance exposes a method to evaluate ``allow`` rules that takes three
arguments, **actor**, **action**, and **resource**:


.. tabs::
  .. group-tab:: Python

    .. literalinclude:: /examples/quickstart/python/allow-01.py
      :language: python
      :lines: 11-13

  .. group-tab:: Ruby

      .. literalinclude:: /examples/quickstart/ruby/allow-01.rb
        :language: ruby
        :lines: 4-6

  .. group-tab:: Java

    .. literalinclude:: /examples/quickstart/java/allow-01.java
      :language: java
      :lines: 6-8
      :dedent: 8

The above method call returns ``true`` if the **actor** ``"alice@example.com"`` may
perform the **action** ``"GET"`` on the
**resource** ``EXPENSES[1]``. We're using ``"GET"`` here to match up with the HTTP
verb used in our server, but this could be anything.

.. note:: For more on **actors**, **actions**, and **resources**, check out
  :doc:`/more/key-concepts`.

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
      >>> oso.is_allowed(alice, "GET", expense)
      False

    We can create a new policy file, and
    explicitly allow Alice to GET expenses...

    .. literalinclude:: /examples/quickstart/polar/expenses-02.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    ...which we can load into our oso instance:

    .. code-block:: pycon

      >>> oso.load_file("expenses.polar")

    ...and now Alice has the power...

    .. code-block:: pycon

      >>> oso.is_allowed(alice, "GET", expense)
      True

    ...and everyone else is still denied:

    .. code-block:: pycon

      >>> OSO.is_allowed("bhavik@example.com", "GET", expense)
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
        irb(main):004:0> OSO.allowed?(actor: alice, action: "GET", resource: expense)
        => false

    We can create a new policy file, and
    explicitly allow Alice to view expenses...

    .. literalinclude:: /examples/quickstart/polar/expenses-02.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    ...which we can load into our oso instance:

    .. code-block:: irb

      irb(main):005:0> OSO.load_file("expenses.polar")
      => #<Set: {"expenses.polar"}>

    ...and now Alice has the power...

    .. code-block:: irb

      irb(main):005:0> OSO.allowed?(actor: "alice", action: "GET", resource: "expense")
      => true

    ...and everyone else is still denied:

    .. code-block:: irb

      irb(main):006:0> OSO.allowed?(actor: "bhavik@example.com", action: "GET", resource: "expense")
      => false

  .. group-tab:: Java

    To follow along, either try using ``jshell`` (requires Java version >= 9)
    or copy the follow code into a ``main`` method in ``Expense.java``.

    Run: ``jshell --class-path oso-0.2.5.jar Expense.java``

    .. tabs::
      .. group-tab:: Java main

          .. code-block:: java
            :caption: :fab:`java` Expense.java

            import com.osohq.oso.Oso;

            public class Expense { 
                // ...

                public static void main(String[] args) throws Exception {
                    Oso oso = new Oso();
                    String alice = "alice@example.com";
                    Expense expense = Expense.EXPENSES[1];
                    System.out.println(oso.isAllowed(alice, "GET", expense));
                }
            }

          Should output:

          .. code-block:: console

            false

      .. group-tab:: JShell

        .. code-block:: jshell

            jshell> import com.osohq.oso.Oso;

            jshell> Oso oso = new Oso();
            oso ==> com.osohq.oso.Oso@55b699ef

            jshell> String alice = "alice@example.com"
            alice ==> "alice@example.com"

            jshell> Expense expense = Expense.EXPENSES[1]
            expense ==> Expense(amount=5000, description=software, submittedBy=alice@example.com)

            jshell> oso.isAllowed(alice, "GET", expense)
            $12 ==> false

    We can create a new policy file, and explicitly allow Alice to view
    expenses

    .. literalinclude:: /examples/quickstart/polar/expenses-02.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    We can load into our oso instance, and then see that Alice has the power and
    everyone else is still denied:

    .. tabs::
      .. group-tab:: Java main

        .. code-block:: java
            :caption: :fab:`java` Expense.java

            public static void main(String[] args) throws Exception {
                Oso oso = new Oso();
                oso.loadFile("expenses.polar");
                String alice = "alice@example.com";
                String bhavik = "bhavik@example.com";
                Expense expense = Expense.EXPENSES[1];
                System.out.println(oso.isAllowed(alice, "GET", expense));
                System.out.println(oso.isAllowed(bhavik, "GET", expense));
            }

        Should output:

        .. code-block:: console

          true
          false

      .. group-tab:: JShell

        .. code-block:: jshell

          jshell> oso.loadFile("expenses.polar")

          jshell> oso.isAllowed(alice, "GET", expense)
          $14 ==> true

          jshell> oso.isAllowed("bhavik@example.com", "GET", expense)
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


Authorizing HTTP requests
=========================

Now that we are confident we can control access to our expense data,
let's see what it would look like in a web server.
Our web server contains some simple logic to filter out bad requests and not much else.

In lieu of setting up real identity and authentication systems, we'll used a
custom HTTP header to indicate that a request is "authenticated" as a particular
user. The header value will be an email address, e.g., ``"alice@example.com"``.
We'll pass it to ``allow`` as the **actor** and we'll use the HTTP method as the
**action**.

Finally, the **resource** is the expense retrieved from our stored expenses.

.. tabs::

  .. group-tab:: Python

    .. literalinclude:: /examples/quickstart/python/server.py
      :class: copybutton
      :caption: :fab:`python` server.py :download:`(link) </examples/quickstart/python/server.py>`
      :language: python
      :emphasize-lines: 26-29

  .. group-tab:: Ruby

    .. literalinclude:: /examples/quickstart/ruby/server.rb
      :class: copybutton
      :caption: :fas:`gem` server.rb :download:`(link) </examples/quickstart/ruby/server.rb>`
      :language: ruby
      :emphasize-lines: 18-21

  .. group-tab:: Java

    .. literalinclude:: /examples/quickstart/java/Server.java
      :class: copybutton
      :caption: :fab:`java` Server.java :download:`(link) </examples/quickstart/java/Server.java>`
      :language: java
      :emphasize-lines: 34-38
    
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

        $ javac -cp oso-0.2.5.jar:. Server.java
        $ java Server
        Server running on /127.0.0.1:5050

...and then, in another terminal, we can test everything works by making some requests:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(amount=500, description='coffee', submitted_by='alice@example.com')
  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Not Authorized!

If you aren't seeing the same thing, make sure you created your policy
correctly in ``expenses.polar``.

Rules over dynamic data
-----------------------

It's nice that Alice can view expenses, but it would be really onerous if
we had to write a separate rule for every single actor we wanted to authorize.
Luckily, we don't!

Let's replace our static rule checking that the provided email matches
``"alice@example.com"`` with a dynamic one that checks that the provided email
ends in ``"@example.com"``. That way, everyone at Example.com, Inc. will be
able to view expenses, but no one outside the company will be able to:

.. tabs::
  .. group-tab:: Python

    .. literalinclude:: /examples/quickstart/polar/expenses-03-py.polar
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

    And that's just the tip of the iceberg. You can register *any* application object with
    oso and then leverage it in your application's authorization policy.
    In the next section, we'll update
    our existing policy to leverage the ``Expense`` class defined in our
    application.


  .. group-tab:: Ruby

    .. literalinclude:: /examples/quickstart/polar/expenses-03-rb.polar
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

    And that's just the tip of the iceberg. You can register *any* application object with
    oso and then leverage it in your application's authorization policy.
    In the next section, we'll update
    our existing policy to leverage the ``Expense`` class defined in our
    application.


  .. group-tab:: Java

    .. literalinclude:: /examples/quickstart/polar/expenses-03-java.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

    .. |string_endsWith| replace:: the ``String.endsWith?`` method
    .. _string_endsWith: https://docs.oracle.com/javase/8/docs/api/java/lang/String.html#endsWith-java.lang.String-

    We bind the provided email to the ``actor`` variable in the rule head and then
    perform the ``.endsWith("@example.com")`` check in the rule body. If you
    noticed that the ``.endsWith`` call looks pretty familiar, you're right on ---
    oso is actually calling out to |string_endsWith|_ defined in the Java standard
    library. The **actor** value passed to oso is a Java string, and oso allows us
    to call any ``String`` method from Java's standard library on it.

    And that's just the tip of the iceberg. You can register *any* application object with
    oso and then leverage it in your application's authorization policy.
    In the next section, we'll update
    our existing policy to leverage the ``Expense`` class defined in our
    application.


Once we've added our new dynamic rule and restarted the web server, every user
with an ``@example.com`` email should be allowed to view any expense:

.. code-block:: console

  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Expense(...)

If a user's email doesn't end in ``"@example.com"``, the rule fails, and they
are denied access:

.. code-block:: console

  $ curl -H "user: bhavik@foo.com" localhost:5050/expenses/1
  Not Authorized!


Writing authorization policy over application data
==================================================

At this point, the higher-ups at Example.com, Inc. are still not satisfied with
our access policy that allows all employees to see each other's expenses. They
would like us to modify the policy such that employees can only see their own
expenses.

To accomplish that, we can replace our existing rule with:

.. tabs::

  .. group-tab:: Python

    .. literalinclude:: /examples/quickstart/polar/expenses-04.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

  .. group-tab:: Ruby

    .. literalinclude:: /examples/quickstart/polar/expenses-04.polar
      :caption: :fa:`oso` expenses.polar
      :class: copybutton

  .. group-tab:: Java

    .. literalinclude:: /examples/quickstart/polar/expenses-04-java.polar
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
  :doc:`/getting-started/policies/application-types`.



Summary
=======

We just blitzed through a ton of stuff:

* Installing oso.
* Setting up our app to enforce the policy decisions made by oso.
* Writing authorization rules over static and dynamic application data.

.. todo::
    Make these actual links.

.. admonition:: What's next
    :class: tip whats-next

    * Explore how to :doc:`/getting-started/application/index`.
    * Dig deeper on :doc:`/getting-started/policies/index`.
    * Check out oso in action: :doc:`/using/examples/index`.
    * Explore the :doc:`/more/design-principles` behind oso.

------------------------

.. admonition:: Sign up for our newsletter!

    .. raw:: html

        <script charset="utf-8" type="text/javascript" src="//js.hsforms.net/forms/shell.js"></script>
        <script>
          hbspt.forms.create({
            portalId: "8091225",
            formId: "109f461f-8b3a-4dfa-a942-fd40b6f6e27f"
        });
        </script>
