===============
Getting started
===============

If you don't know what oso is, head back to `the introduction </>`_. If you've
never used oso before and want to get up-and-running quickly, this guide is for
you.

In general, it takes less than 5 minutes to add oso to an existing application
and begin writing an authorization policy. In the next 15 minutes, we're going
to create a simple web server with no authorization, add oso to the project,
and then write our first policy. We encourage you to code along in your local
environment!

First, let's create a simple web server:

.. literalinclude:: /examples/getting-started/server-01.rb
  :caption: server.rb
  :language: ruby
  :lines: 3-

Our server currently has no authorization --- anyone is able to view our
precious secrets. Let's use `cURL <https://curl.haxx.se/>`_ to check that
everything's working. We'll first start our server...

.. code-block:: console

  $ ruby server.rb
  [2020-07-15 00:35:52] INFO  WEBrick 1.3.1
  [2020-07-15 00:35:52] INFO  ruby 2.4.10 (2020-03-31) [x86_64-linux]
  [2020-07-15 00:35:52] INFO  WEBrick::HTTPServer#start: pid=537647 port=5050

...and then, in another terminal, make a request to our running server:

.. code-block:: console

  $ curl localhost:5050
  Authorized!

Adding oso
==========

.. |gem| replace:: the ``oso-oso`` gem
.. _gem: https://rubygems.org/gems/oso-oso

Next, let's add oso to our application so that we can write our first
authorization policy. If you don't already have |gem|_ installed, go ahead and
install it now:

.. code-block:: console

  $ gem install oso-oso
  Fetching oso-oso-0.2.0.gem
  Successfully installed oso-oso-0.2.0
  1 gem installed

Now that we've installed the gem, we can import it into our project and
construct a new ``Oso`` instance that will serve as our Grand Arbiter of
Authorization:

.. literalinclude:: /examples/getting-started/server-02.rb
  :caption: server.rb
  :language: ruby
  :lines: 3-8

We're now at a point where we can start asking our global ``Oso`` instance to
make authorization decisions. Let's give it a whirl!

Decisions, decisions...
=======================

The ``Oso`` instance exposes an ``allow()`` predicate method that takes three
keyword arguments, **actor**, **action**, and **resource**:

.. literalinclude:: /examples/getting-started/allow-01.rb
  :language: ruby
  :lines: 6

The above method call returns ``true`` if **actor** may perform **action** on
**resource** and ``false`` otherwise.

.. note:: For more on actors, actions, and resources, check out
  :doc:`/auth-fundamentals`.

oso's authorization system is deny-by-default. Since we haven't yet written any
policy code, Alice is not allowed to approve expenses. To see that in action,
start an IRB session and follow along:

.. code-block:: irb

  irb(main):001:0> require 'oso'
  => true
  irb(main):002:0> OSO ||= Oso.new
  => #<Oso::Oso:0x000055a708eb8f70 ...>
  irb(main):003:0> OSO.allow(actor: 'alice', action: 'approve', resource: 'expense')
  => false

We can add a rule explicitly allowing Alice to approve expenses...

.. code-block:: irb

  irb(main):004:0> OSO.load_str <<~RULE
  irb(main):005:0" allow("alice", "approve", "expense");
  irb(main):006:0" RULE
  => nil

...and now Alice has the power...

.. code-block:: irb

  irb(main):007:0> OSO.allow(actor: 'alice', action: 'approve', resource: 'expense')
  => true

...and everyone else is still denied:

.. code-block:: irb

  irb(main):008:0> OSO.allow(actor: 'bhavik', action: 'approve', resource: 'expense')
  => false

.. note:: For a deeper introduction to writing authorization rules with oso,
  see :doc:`/auth-fundamentals`.

Authorizing HTTP requests
=========================

oso produces authorization decisions but makes no assumptions about how those
decisions are enforced. To enforce the authorization decisions returned by
``Oso#allow``, let's create a helper method that we can use in our HTTP handler
to determine whether a request is authorized:

.. literalinclude:: /examples/getting-started/server-03.rb
  :caption: server.rb
  :language: ruby
  :lines: 3-
  :emphasize-lines: 6-8, 12

Our new ``authorize?`` method passes data from the incoming request to ``Oso#allow``:

* The **actor** is pulled from the ``user`` HTTP header.
* The **action** is the HTTP method.
* The **resource** is the request path.

Since we haven't yet added any rules to our server's ``Oso`` instance, all
requests will currently be denied. We can test that out by restarting our
server and making a new request. If we receive an ``Unauthorized!`` response,
everything's working:

.. code-block:: console

  $ curl localhost:5050
  Unauthorized!

As a final step, let's write a couple authorization rules over HTTP requests:

.. literalinclude:: /examples/getting-started/server-04.rb
  :caption: server.rb
  :language: ruby
  :lines: 8-17

And let's test out our new rules:

.. code-block:: console

  $ curl -H "user: alice@example.com" localhost:5050/anything
  Authorized!
  $ curl -H "user: bhavik@example.com" -d '' localhost:5050/admin
  Authorized!

We encourage you to experiment with adding your own rules to the policy!

Summary
=======

We just blitzed through a ton of stuff:

* Installing oso.
* Setting up our app to enforce the policy decisions made by oso.
* Writing new authorization rules.

If you're interested in what sets oso apart from existing authorization
solutions, check out :doc:`/why-oso`. If you want to learn more about
authorization in oso, including common patterns like :doc:`/auth-models/rbac`
and :doc:`/auth-models/abac`, we recommend continuing on to the
:doc:`/auth-fundamentals` guide. For more details on the logic programming
language we used to write our authorization policies, head on over to the
:doc:`/language/index` guide.
