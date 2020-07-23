===================
Add oso to your app
===================
.. todo::
    Turn this into the "Getting Started Level 2 guide"

In this guide, we'll take you through how to add oso to your
application. We'll focus on common patterns for where to put authorization, and authorizing
access to data.

We have a sample expenses application built with Flask that we'll be using for the tutorial.
Clone this from here:

.. todo:: Insert little github snippet box + actually have a real link

:fab:`github` `osohq/oso-flask-tutorial <https://gitbub.com/osohq/oso-flask>`_

Our expenses application reads from a sqlite database, and has a few simple endpoints for returning
results. We encourage you to take a look around before continuing!

.. literalinclude:: /examples/application/expenses-flask/app/expense.py
    :caption: :fab:`python` expense.py
    :language: python
    :lines: 29-31

Running the example
-------------------

The example comes with a SQL database dump, which you can initialize with:

.. code-block:: console
    :class: copybutton

    $ sqlite3 expenses.db ".read expenses.sql"

The application has a few requirements, including Flask and, or course, oso.
We recommend running this with a virtual environment:

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
    :lines: 12-19

Now that we have this in place, we can write a simple policy to allow anyone
to call our index route, and see the hello message:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :language: polar
    :lines: 1-2

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
    :lines: 16-24

.. literalinclude:: /examples/application/expenses-flask/app/user.py
    :caption: :fab:`python` user.py
    :language: python
    :lines: 43-44

So we can use :doc:`type checking </using/policies/application-types>`
to only allow the request when the user is a ``User``:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :language: polar
    :lines: 4-5


.. code-block:: console

    $ curl localhost:5000/whoami
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Not Authorized!</p>

    $ curl -H "user: alice@foo.com"  localhost:5000/whoami
    You are alice@foo.com, the CEO at Foo Industries

Authorizing Access to Data
--------------------------

Since we have access to the full user object in our authorization handler,
we are able to make allow decisions based on that information.

We could go a step further and use this to 




Things to cover:

- Data-access authorization, means we can perform authorization when fetching data in our application
- Combine the two, we can make sure authorization took place:
    e.g. check that authorization took place at some point on the request path.


.. toctree::
    :hidden:

    Guide <self>
    Patterns <patterns>
