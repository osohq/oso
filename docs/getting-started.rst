===============
Getting Started
===============

If you don't know what oso is, head back to `the introduction </>`_. If you've
never used oso before and want to get up-and-running quickly, this guide is for
you.

In general, it takes less than 5 minutes to add oso to an existing application
and begin writing an authorization policy. In the next 15 minutes, we're going
to create a simple web server with no authorization, add oso to the project,
and then write our first policy. We encourage you to code along in your local
environment!

A basic web server
==================

Our application serves data about expenses submitted by users. The ``Expense``
class isn't too complicated...

.. .. literalinclude:: /examples/getting-started/ruby/server-01.rb
..   :caption: server.rb
..   :language: ruby
..   :lineno-start: 1
..   :lines: 1-9

.. literalinclude:: /examples/getting-started/python/server-01.py
  :caption: server.py
  :language: python
  :lineno-start: 1
  :lines: 1-7

...and our "database" of expenses is a map from ID to expense:

.. .. literalinclude:: /examples/getting-started/ruby/server-01.rb
..   :caption: server.rb
..   :language: ruby
..   :lineno-start: 9
..   :lines: 9-13

.. literalinclude:: /examples/getting-started/python/server-01.py
  :caption: server.py
  :language: python
  :lineno-start: 9
  :lines: 9-13

Our web server contains some simple logic to filter out bad requests and not much else:

.. .. literalinclude:: /examples/getting-started/ruby/server-01.rb
..   :caption: server.rb
..   :language: ruby
..   :lineno-start: 17
..   :lines: 17-

.. literalinclude:: /examples/getting-started/python/server-01.py
  :caption: server.py
  :language: python
  :lineno-start: 15
  :lines: 15-

If the request path matches the form ``/expenses/:id`` and ``:id`` is the ID of
an existing expense, we respond with the expense data. Otherwise, we return
``"Not Found!"``.

Let's use `cURL <https://curl.haxx.se/>`_ to check that everything's working.
We'll first start our server...

.. .. code-block:: console

..   $ ruby server.rb
..   [2020-07-15 00:35:52] INFO  WEBrick 1.3.1
..   [2020-07-15 00:35:52] INFO  ruby 2.4.10 (2020-03-31) [x86_64-linux]
..   [2020-07-15 00:35:52] INFO  WEBrick::HTTPServer#start: pid=537647 port=5050

.. code-block:: console

  $ python server.py
  running on port 5050

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

.. |gem| replace:: the **oso-oso** gem
.. _gem: https://rubygems.org/gems/oso-oso

.. |python-package| replace:: the **oso** package
.. _python-package: https://pypi.org/project/oso/

.. In order to write our first authorization policy, we first need to add oso to
.. our application. If you don't already have |python-package|_ installed, go ahead and
.. install it now:

In order to write our first authorization policy, we first need to add oso to
our application. If you don't already have |python-package|_ installed, go
ahead and install it now:

.. TODO: replace the hard-coded version number in the below snippet with the
   current latest version on RubyGems... somehow.

.. .. code-block:: console

..   $ gem install oso-oso
..   Fetching oso-oso-#.#.#.gem
..   Successfully installed oso-oso-#.#.#
..   1 gem installed

.. code-block::

  $ pip install -q oso==0.2.2

Now that we've installed oso, we can import it into our project and construct
a new ``Oso`` instance that will serve as our authorization engine:

.. .. literalinclude:: /examples/getting-started/ruby/server-02.rb
..   :caption: server.rb
..   :language: ruby
..   :lines: 17-23
..   :lineno-start: 15

.. literalinclude:: /examples/getting-started/python/server-02.py
  :caption: server.py
  :language: python
  :lines: 17-23
  :lineno-start: 15

And just like that, we're ready to start asking our global ``Oso`` instance to
make authorization decisions!

Decisions, decisions...
=======================

.. The ``Oso`` instance exposes an ``allow()`` predicate method that takes three
.. keyword arguments, **actor**, **action**, and **resource**:

The ``Oso`` instance exposes an ``allow()`` predicate method that takes three
arguments, **actor**, **action**, and **resource**:

.. .. literalinclude:: /examples/getting-started/ruby/allow-01.rb
..   :language: ruby
..   :lines: 4

.. literalinclude:: /examples/getting-started/python/allow-01.py
  :language: python
  :lines: 12

The above method call returns ``true`` if the **actor** ``"alice"`` may
perform the **action** ``"view"`` on the
**resource** ``"expense"``.

.. note:: For more on **actors**, **actions**, and **resources**, check out
  :doc:`/auth-fundamentals`.

oso's authorization system is deny-by-default. Since we haven't yet written any
policy code, Alice is not allowed to view expenses. To see that in action,
start an IRB session and follow along:

.. .. code-block:: irb
..
..  irb(main):001:0> require "oso"
..  => true
..  irb(main):002:0> OSO ||= Oso.new
..  => #<Oso::Oso:0x000055a708eb8f70 ...>
..  irb(main):003:0> OSO.allow(actor: "alice", action: "view", resource: "expense")
..  => false

.. code-block:: pycon

  >>> from oso import Oso
  >>> OSO = Oso()
  >>> OSO
  <oso.Oso object at 0x7f267494dc70>
  >>> OSO.allow("alice", "view", "expense")
  False

We can add a rule explicitly allowing Alice to view expenses...

.. .. code-block:: irb
.. 
..   irb(main):004:0> OSO.load_str 'allow("alice", "view", "expense");'
..   => nil

.. code-block:: pycon

  >>> OSO.load_str('allow("alice", "view", "expense");')

...and now Alice has the power...

.. .. code-block:: irb
.. 
..   irb(main):005:0> OSO.allow(actor: "alice", action: "view", resource: "expense")
..   => true

.. code-block:: pycon

  >>> OSO.allow("alice", "view", "expense")
  True

...and everyone else is still denied:

.. .. code-block:: irb
.. 
..   irb(main):006:0> OSO.allow(actor: "bhavik", action: "view", resource: "expense")
..   => false

.. code-block:: pycon

  >>> OSO.allow("bhavik", "view", "expense")
  False

When we ask oso for a policy decision via ``Oso#allow``, the oso engine
searches through its knowledge base to determine whether the provided
**actor**, **action**, and **resource** satisfy any **allow** rules.

In the above case, we passed in ``"alice"`` as the **actor**, ``"view"`` as the
**action**, and ``"expense"`` as the **resource**, satisfying the
``allow("alice", "view", "expense");`` rule. When we pass in ``"bhavik"`` as
the actor, the rule no longer succeeds because the string ``"bhavik"`` does not
match the string ``"alice"``.

.. note:: For a deeper introduction to writing authorization rules with oso,
  see :doc:`/auth-fundamentals`.

Authorizing HTTP requests
=========================

In lieu of setting up real identity and authentication systems, we're going to
use a custom HTTP header to indicate that a request is "authenticated" as a
particular user. The header value will be an email address, e.g.,
``"alice@example.com"``. We'll pass it to ``Oso#allow`` as the **actor**...

.. .. literalinclude:: /examples/getting-started/ruby/server-03.rb
..   :caption: server.rb
..   :language: ruby
..   :lineno-start: 24
..   :lines: 24-25
..   :emphasize-lines: 2

.. literalinclude:: /examples/getting-started/python/server-03.py
  :caption: server.py
  :language: python
  :lineno-start: 27
  :lines: 27-28
  :emphasize-lines: 2


...and we'll use the HTTP method as the **action**:

.. .. literalinclude:: /examples/getting-started/ruby/server-03.rb
..   :caption: server.rb
..   :language: ruby
..   :lineno-start: 24
..   :lines: 24-28
..   :emphasize-lines: 3

.. literalinclude:: /examples/getting-started/python/server-03.py
  :caption: server.py
  :language: python
  :lineno-start: 27
  :lines: 27-32
  :emphasize-lines: 3

To recap:

* The **actor** is pulled from the ``user`` HTTP header.
* The **action** is the HTTP method.
* The **resource** is the expense retrieved from our "database".

If we pass all three pieces of data to ``Oso#allow``, it'll return a boolean
decision that we can use in our server's response logic:

.. .. literalinclude:: /examples/getting-started/ruby/server-03.rb
..   :caption: server.rb
..   :language: ruby
..   :lineno-start: 24
..   :lines: 24-37
..   :emphasize-lines: 9-13

.. literalinclude:: /examples/getting-started/python/server-03.py
  :caption: server.py
  :language: python
  :lineno-start: 27
  :lines: 27-43
  :emphasize-lines: 11-14

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

.. .. literalinclude:: /examples/getting-started/ruby/server-04.rb
..   :caption: server.rb
..   :language: ruby
..   :lines: 19-20
..   :lineno-start: 19
..   :emphasize-lines: 2

.. literalinclude:: /examples/getting-started/python/server-04.py
  :caption: server.py
  :language: python
  :lines: 17-18
  :lineno-start: 17
  :emphasize-lines: 2

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

.. .. literalinclude:: /examples/getting-started/ruby/server-05.rb
..   :caption: server.rb
..   :language: ruby
..   :lines: 19-23
..   :lineno-start: 19
..   :emphasize-lines: 2-

.. literalinclude:: /examples/getting-started/python/server-05.py
  :caption: server.py
  :language: python
  :lines: 17-21
  :lineno-start: 17
  :emphasize-lines: 2-

.. .. |string_end_with| replace:: the ``String#end_with?`` method
.. .. _string_end_with: https://ruby-doc.org/core/String.html#method-i-end_with-3F

.. |string_end_with| replace:: the ``str#endswith`` method
.. _string_end_with: https://docs.python.org/3/library/stdtypes.html#str.endswith

.. We bind the provided email to the ``actor`` variable in the rule head and then
.. perform the ``.end_with?("@example.com")`` check in the rule body. If you
.. noticed that the ``.end_with?`` call looks pretty familiar, you're right on ---
.. oso is actually calling out to |string_end_with|_ defined in the Ruby standard
.. library. The **actor** value passed to oso is a Ruby string, and oso allows us
.. to call any ``String`` method from Ruby's standard library on it.
.. 
.. And that's just the tip of the iceberg. You can register *any* Ruby object with
.. oso and then leverage it in your application's authorization policy. For
.. example, if you have ``Expense`` and ``User`` classes defined in your
.. application, you could write a policy rule in oso that says a ``User`` may
.. approve an ``Expense`` if they manage the ``User`` who submitted the expense
.. and the expense's amount is less than $100.00:

We bind the provided email to the ``actor`` variable in the rule head and then
perform the ``.endswith("@example.com")`` check in the rule body. If you
noticed that the ``.endswith`` call looks pretty familiar, you're right on ---
oso is actually calling out to |string_end_with|_ defined in the Python standard
library. The **actor** value passed to oso is a Python string, and oso allows us
to call any ``str`` method from Python's standard library on it.

And that's just the tip of the iceberg. You can register *any* Python object with
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

.. .. literalinclude:: /examples/getting-started/ruby/server-05.rb
..   :caption: server.rb
..   :language: ruby
..   :lines: 19-23
..   :lineno-start: 19
..   :emphasize-lines: 2-

.. literalinclude:: /examples/getting-started/python/server-05.py
  :caption: server.py
  :language: python
  :lines: 17-21
  :lineno-start: 17
  :emphasize-lines: 2-

.. .. |string_end_with| replace:: the ``String#end_with?`` method
.. .. _string_end_with: https://ruby-doc.org/core/String.html#method-i-end_with-3F

.. |string_end_with| replace:: the ``str#endswith`` method
.. _string_end_with: https://docs.python.org/3/library/stdtypes.html#str.endswith

.. We bind the provided email to the ``actor`` variable in the rule head and then
.. perform the ``.end_with?("@example.com")`` check in the rule body. If you
.. noticed that the ``.end_with?`` call looks pretty familiar, you're right on ---
.. oso is actually calling out to |string_end_with|_ defined in the Ruby standard
.. library. The **actor** value passed to oso is a Ruby string, and oso allows us
.. to call any ``String`` method from Ruby's standard library on it.
.. 
.. And that's just the tip of the iceberg. You can register *any* Ruby object with
.. oso and then leverage it in your application's authorization policy. For
.. example, if you have ``Expense`` and ``User`` classes defined in your
.. application, you could write a policy rule in oso that says a ``User`` may
.. approve an ``Expense`` if they manage the ``User`` who submitted the expense
.. and the expense's amount is less than $100.00:

We bind the provided email to the ``actor`` variable in the rule head and then
perform the ``.endswith("@example.com")`` check in the rule body. If you
noticed that the ``.endswith`` call looks pretty familiar, you're right on ---
oso is actually calling out to |string_end_with|_ defined in the Python standard
library. The **actor** value passed to oso is a Python string, and oso allows us
to call any ``str`` method from Python's standard library on it.

And that's just the tip of the iceberg. You can register *any* Python object with
oso and then leverage it in your application's authorization policy. For
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
  :doc:`application-library/application-types`.

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

To accomplish that, we can extend our existing rule with a second condition:

.. .. literalinclude:: /examples/getting-started/ruby/server-06.rb
..   :caption: server.rb
..   :language: ruby
..   :lines: 19-24
..   :emphasize-lines: 3, 5
..   :lineno-start: 19

.. literalinclude:: /examples/getting-started/python/server-06.py
  :caption: server.py
  :language: python
  :lines: 17-22
  :emphasize-lines: 3, 5
  :lineno-start: 17

Behind the scenes, oso looks up the ``submitted_by`` field on the provided
``Expense`` instance and compares that value against the provided **actor**.
And just like that, an actor can only see an expense if they have an
``@example.com`` email *and* they submitted the expense.

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
solutions, check out :doc:`/why-oso`. If you want to learn more about
authorization in oso, including common patterns like :doc:`/auth-models/rbac`
and :doc:`/auth-models/abac`, we recommend continuing on to the
:doc:`/auth-fundamentals` guide. For more details on the logic programming
language we used to write our authorization policies, head on over to the
:doc:`/language/index` guide.
