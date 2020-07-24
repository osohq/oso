===================
Add oso to your app
===================
.. todo::
    Turn this into the "Getting Started Level 2 guide"

This guide covers a little more detail about how to add oso to your application.

Whereas in the :doc:`/getting-started/quickstart` we zoomed through an example
of authorization in a to server, in this guide we'll show some more practical
examples in the context of a more realistic Python application.

Our sample expenses application is built with Flask. To follow along, or
dig into the code, clone it from here:

.. todo:: Insert little github snippet box + actually have a real link

:fab:`github` `osohq/oso-flask-tutorial <https://gitbub.com/osohq/oso-flask>`_

Our expenses application reads from a sqlite database, and has a few simple endpoints for returning
results. We encourage you to take a look around before continuing!

.. literalinclude:: /examples/application/expenses-flask/app/expense.py
    :caption: :fab:`python` `expense.py <https://gitbub.com/osohq/oso-flask/tree/main/app/expense.py>`_
    :language: python
    :lines: 51-54

Running the example
-------------------

The example comes with a SQL database dump, which you can load with:

.. code-block:: console
    :class: copybutton

    $ sqlite3 expenses.db ".read expenses.sql"

The application has a few requirements, including Flask and, or course, oso.
We recommend installing within a virtual environment:

.. code-block:: console
    :class: copybutton

    $ python3 -m venv venv
    $ source venv/bin/activate
    $ pip3 install -r requirements.txt
    $ FLASK_ENV=development flask run --extra-files app/authorization.polar

Authorizing Routes
--------------------

The first thing we might want to add to our application is some simple authorization
to allow some users to only have access to certain routes if they are logged in.

We can apply apply authorization to **every** incoming request by setting up
a simple ``before_app_request`` hook and using ``oso.allow``:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.py
    :caption: :fab:`python` authorization.py
    :language: python
    :lines: 9-17

Now that this is in place, we can write a simple policy to allow anyone
to call our index route, and see the hello message:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :language: polar
    :lines: 3-4

.. code-block:: console

    $ curl localhost:5000/
    hello Guest
    $ curl -H "user: alice@foo.com"  localhost:5000/
    hello alice@foo.com


But we also have a ``/whoami`` route that only properly authenticated users can
see.

We have two different user types here: the ``Guest`` class and the ``User`` class.
The latter corresponds to users who have "authenticated" by supplying a valid
email address in the request header.


.. literalinclude:: /examples/application/expenses-flask/app/user.py
    :caption: :fab:`python` user.py
    :language: python
    :lines: 16-25

.. literalinclude:: /examples/application/expenses-flask/app/user.py
    :caption: :fab:`python` user.py
    :language: python
    :lines: 55-56

So we can use :doc:`type checking </using/policies/application-types>`
to only allow the request when the user is a ``User``:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :language: polar
    :lines: 6-7


.. code-block:: console

    $ curl localhost:5000/whoami
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Not Authorized!</p>

    $ curl -H "user: alice@foo.com"  localhost:5000/whoami
    You are alice@foo.com, the CEO at Foo Industries. (User ID: 1)

.. tip::

    Interested in understanding more about what is happening here?
    Check out the :doc:`/using/examples/user_types` example.


Authorizing Access to Data
--------------------------

In the :doc:`/getting-started/quickstart`, our main objective was to
determine who could "GET" expenses. Our final policy looked like:

.. literalinclude:: /examples/quickstart/polar/expenses-04.polar
    :caption: :fa:`oso` expenses.polar

In our expenses sample application, we have something similar:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :lines: 20-22,26-27

But we've rewritten the policy to use a new ``submitted`` predicate in case we want
to change the logic in the future.

Although we *could* implement our 

.. todo::
    Revisit the example from quickstart
    Accessing expenses in different ways?
    E.g. partial expense data, full expense data

    Goals:
    - Go a little more in depth than the quickstart
    - We don't actually have a real best practice to apply yet
    - Show how to control access to data (in the data acccess layer?)
    - Tee up the patterns doc

    is it based on the page, do model layer,
    call allow in the route handler


Things to cover:

- Data-access authorization, means we can perform authorization when fetching data in our application
- Combine the two, we can make sure authorization took place:
    e.g. check that authorization took place at some point on the request path.


.. toctree::
    :hidden:

    Guide <self>
    patterns
