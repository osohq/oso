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
      :caption: Download: :download:`server.py </examples/getting-started/python/server.py>`
      :language: python

  .. group-tab:: Ruby

    .. literalinclude:: /examples/getting-started/ruby/server.rb
      :class: copybutton
      :caption: Download: :download:`server.rb </examples/getting-started/ruby/server.rb>`
      :language: ruby


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

Now that we've installed oso, we can import it into our project and construct
a new ``Oso`` instance that will serve as our authorization engine:


.. tabs::
  .. group-tab:: Python

    .. literalinclude:: server-02.py
      :base_path: /examples/getting-started/python/
      :filename: server.py
      :diff: server.py

  .. group-tab:: Ruby

      .. literalinclude:: server-02.rb
        :base_path: /examples/getting-started/ruby/
        :filename: server.rb
        :diff: server.rb


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
      :lines: 12

  .. group-tab:: Ruby

      .. literalinclude:: /examples/getting-started/ruby/allow-01.rb
        :language: ruby
        :lines: 4

The above method call returns ``true`` if the **actor** ``"alice"`` may
perform the **action** ``"view"`` on the
**resource** ``"expense"``.

.. note:: For more on **actors**, **actions**, and **resources**, check out
  :doc:`/using/auth-fundamentals`.

oso's authorization system is deny-by-default. Since we haven't yet written any
policy code, Alice is not allowed to view expenses. To see that in action,
start a REPL session and follow along:

.. tabs::
  .. group-tab:: Python

    .. code-block:: pycon

      >>> from oso import Oso
      >>> OSO = Oso()
      >>> OSO
      <oso.Oso object at 0x7f267494dc70>
      >>> OSO.allow("alice", "view", "expense")
      False


    We can add a rule explicitly allowing Alice to view expenses...

    .. code-block:: pycon

      >>> OSO.load_str('allow("alice", "view", "expense");')

    ...and now Alice has the power...

    .. code-block:: pycon

      >>> OSO.allow("alice", "view", "expense")
      True

    ...and everyone else is still denied:

    .. code-block:: pycon

      >>> OSO.allow("bhavik", "view", "expense")
      False


  .. group-tab:: Ruby

    .. code-block:: irb

      irb(main):001:0> require "oso"
      => true
      irb(main):002:0> OSO ||= Oso.new
      => #<Oso::Oso:0x000055a708eb8f70 ...>
      irb(main):003:0> OSO.allow(actor: "alice", action: "view", resource: "expense")
      => false

    We can add a rule explicitly allowing Alice to view expenses...

    .. code-block:: irb

      irb(main):004:0> OSO.load_str 'allow("alice", "view", "expense");'
      => nil

    ...and now Alice has the power...

    .. code-block:: irb

      irb(main):005:0> OSO.allow(actor: "alice", action: "view", resource: "expense")
      => true

    ...and everyone else is still denied:

    .. code-block:: irb

      irb(main):006:0> OSO.allow(actor: "bhavik", action: "view", resource: "expense")
      => false

When we ask oso for a policy decision via ``allow``, the oso engine
searches through its knowledge base to determine whether the provided
**actor**, **action**, and **resource** satisfy any **allow** rules.

In the above case, we passed in ``"alice"`` as the **actor**, ``"view"`` as the
**action**, and ``"expense"`` as the **resource**, satisfying the
``allow("alice", "view", "expense");`` rule. When we pass in ``"bhavik"`` as
the actor, the rule no longer succeeds because the string ``"bhavik"`` does not
match the string ``"alice"``.

.. note:: For a deeper introduction to writing authorization rules with oso,
  see :doc:`/using/auth-fundamentals`.

Authorizing HTTP requests
=========================

In lieu of setting up real identity and authentication systems, we're going to
use a custom HTTP header to indicate that a request is "authenticated" as a
particular user. The header value will be an email address, e.g.,
``"alice@example.com"``. We'll pass it to ``allow`` as the **actor**
and we'll use the HTTP method as the **action**.

Finally, the **resource** is the expense retrieved from our stored expenses.

If we pass all three pieces of data to ``allow``, it'll return a boolean
decision that we can use in our server's response logic:

.. tip::
  Try copying the patch, and applying it locally with:

  .. code-block::

      $ patch <<EOF <hit enter>
      > <paste contents>
      EOF

.. tabs::
  .. group-tab:: Python

    .. literalinclude:: server-03.py
      :base_path: /examples/getting-started/python/
      :filename: server.py
      :diff: server-02.py
      :class: copybutton

    .. literalinclude:: expenses-01.pol
      :base_path: /examples/getting-started/polar/
      :caption: expenses.pol

  .. group-tab:: Ruby

    .. literalinclude:: server-03.rb
      :base_path: /examples/getting-started/ruby/
      :filename: server.rb
      :diff: server-02.rb
      :class: copybutton

    .. literalinclude:: expenses-01.pol
      :base_path: /examples/getting-started/polar/
      :caption: expenses.pol


Since we haven't yet added any authorization rules to our server's ``Oso``
instance, all requests for valid expenses will be denied. We can test that by
restarting the server and making a valid request. If you receive a ``Not
Authorized!`` response, everything's working:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Not Authorized!

Rules over static data
----------------------

A web server that only ever returns ``Not Authorized!`` isn't a ton of fun, so
let's write a rule allowing certain HTTP requests and load it into our ``Oso``
instance's knowledge base.

Our first rule allows the actor ``"alice@example.com"`` to ``GET`` any expense:

.. literalinclude:: expenses-02.pol
  :base_path: /examples/getting-started/polar/
  :caption: expenses.pol
  :class: copybutton

The rule will succeed if the **actor** and **action** match the strings
``"alice@example.com"`` and ``"GET"``, respectively. We capture the provided
**resource** value in the ``_expense`` variable, but we don't do anything with
it since we want the rule to apply to all expenses.

With the first rule in place, Alice can ``GET`` expenses:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/expenses/1
  Expense(...)
  $ curl -H "user: alice@example.com" localhost:5050/expenses/3
  Expense(...)

But Bhavik can't since their email doesn't match the string
``"alice@example.com"``:

.. code-block:: console

  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/1
  Not Authorized!
  $ curl -H "user: bhavik@example.com" localhost:5050/expenses/3
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

    .. literalinclude:: expenses-03-py.pol
      :base_path: /examples/getting-started/polar/
      :caption: expenses.pol
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

    .. literalinclude:: expenses-03-rb.pol
      :base_path: /examples/getting-started/polar/
      :caption: expenses.pol
      :class: copybutton

    .. |string_end_with| replace:: the ``String#end_with?`` method
    .. _string_end_with: https://ruby-doc.org/core/String.html#method-i-end_with-3F

    We bind the provided email to the ``actor`` variable in the rule head and then
    perform the ``.end_with?("@example.com")`` check in the rule body. If you
    noticed that the ``.end_with?`` call looks pretty familiar, you're right on ---
    oso is actually calling out to |string_end_with|_ defined in the Ruby standard
    library. The **actor** value passed to oso is a Ruby string, and oso allows us
    to call any ``String`` method from Ruby's standard library on it.

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

For
example, if you have ``Expense`` and ``User`` classes defined in your
application, you could write a policy rule in oso that says a ``User`` may
approve an ``Expense`` if they manage the ``User`` who submitted the expense
and the expense's amount is less than $100.00:


.. code-block:: polar

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


Writing your access policy as declarative rules over your app's classes and
data is one of oso's most powerful features. In the next section, we'll update
our existing policy to leverage the ``Expense`` class defined in our
application.

Writing authorization policy over application data
==================================================

At this point, the higher-ups at Example.com, Inc. are still not satisfied with
our access policy that allows all employees to see each other's expenses. They
would like us to modify the policy such that employees can only see their own
expenses.

To accomplish that, we can replace our existing rule with:

.. literalinclude:: expenses-04.pol
  :base_path: /examples/getting-started/polar/
  :caption: expenses.pol
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
:doc:`/using/auth-fundamentals` guide. For more details on the logic programming
language we used to write our authorization policies, head on over to the
:doc:`/understand/language/polar-fundamentals` guide.
