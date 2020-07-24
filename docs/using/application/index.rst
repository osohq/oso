===================
Add oso to your app
===================

This guide covers a little more detail about how to add oso to your application.

Whereas in the :doc:`/getting-started/quickstart` we zoomed through an example
of authorization in a to server, in this guide we'll show some more practical
examples in the context of a more realistic Python application.

Our sample expenses application is built with Flask. To follow along, or
dig into the code, clone it from here:

.. todo:: Insert little github snippet box + actually have a real link

:fab:`github` `osohq/oso-flask-tutorial <https://github.com/osohq/oso-flask-tutorial>`_

Our expenses application reads from a sqlite database, and has a few simple endpoints for returning
results. We encourage you to take a look around before continuing!

.. literalinclude:: /examples/application/expenses-flask/app/expense.py
    :caption: :fab:`python` `expense.py <https://github.com/osohq/oso-flask-tutorial/tree/main/app/expense.py>`_
    :language: python
    :lines: 49-51

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
    :lines: 52-53

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

In our expenses sample application, we have something similar,
but we've rewritten the policy to use a new ``submitted`` predicate in case we want
to change the logic in the future.

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :lines: 21-25


To handle authorizing access to data, we've implemented a little helper method
for us to use throughout the application:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.py
    :caption: :fab:`python` authorization.py
    :language: python
    :lines: 20-25

... so authorizing the GET request looks like:

.. literalinclude:: /examples/application/expenses-flask/app/expense.py
    :caption: :fab:`python` expense.py
    :language: python
    :lines: 49-52

.. tip::

    We *could* implement this in our request layer authorization that we already have. We could
    even call the ``Expense.lookup`` class method to get the expense so we can check attributes
    for the authorization decision.
    There are a few reasons that may not be the best idea, which we wont cover here but
    those interested could go read about this in **TODO**.

    .. todo:: Add a document?

Let's give it a try!

.. code-block:: console

    $ curl localhost:5000/expenses/2
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Not Authorized!</p>

    $ curl -H "user: alice@foo.com" localhost:5000/expenses/2
    Expense(amount=17743, description='Pug irony.', user_id=1, id=2)


This pattern is pretty convenient. We can easily apply it elsewhere:

.. literalinclude:: /examples/application/expenses-flask/app/organization.py
    :caption: :fab:`python` organization.py
    :language: python
    :lines: 29-32

.. code-block:: console

    $ curl -H "user: alice@foo.com" localhost:5000/organizations/1
    Organization(name='Foo Industries', id=1)

    $ curl -H "user: alice@foo.com" localhost:5000/organizations/2
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Not Authorized!</p>



Your turn
---------

We currently have a route with no authorization - the submit endpoint.
We have a rule stating that anyone can PUT to the submit endpoint, but
we want to make sure only authorized expenses are submitted.


.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :lines: 18

.. literalinclude:: /examples/application/expenses-flask/app/expense.py
    :caption: :fab:`python` expense.py
    :language: python
    :lines: 55-63

Right now you can see that anyone can submit an expense:


.. code-block:: console

    $ curl -H "user: alice@foo.com" -X PUT -d '{"amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
    Expense(amount=100, description='Gummy Bears', user_id=1, id=108)


How might we use the ``authorize`` method from before, to make sure that
we check the user is allowed to ``create`` this expense?

We would like to do the authorization on the full ``Expense`` object,
but before it is persisted to the database, so perhaps between these two
lines:

.. literalinclude:: /examples/application/expenses-flask/app/expense.py
    :caption: :fab:`python` expense.py
    :language: python
    :lines: 55-63
    :emphasize-lines: 8-9

We could change the first highlighted line to:

.. code-block:: python

    expense = authorize("create", Expense(**expense_data))

This checks the current user is authorized to create the expense.
If this passes, then we can happily move on to the ``expense.save()``.

Now, nobody will be able to submit expenses, since we haven't yet
added any rules saying they can.

.. admonition:: Add a new rule

    Try editing ``authorization.polar`` to add a rule saying that
    a user can create an expense for which they are assigned as the
    submitter of the expense.

Once you have it working, you can test it by verifying as follows:

.. code-block:: console

    $ curl -H "user: alice@foo.com" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
    Expense(amount=100, description='Gummy Bears', user_id=1, id=111)

    $ curl -H "user: alice@foo.com" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Not Authorized!</p>

We might even consider going a step further and checking that the user is
also authorized to view the *returned* expense, by adding another
``authorize`` check on ``"read"`` for the returned expense.

Summary
-------

In this guide, we showed a few example of how to add oso to an more realistic
application. We added some route-level authorization to control who is allowed
to make requests to certain routes. We also used a new ``authorize`` method to
make it convenient to add data access controls to our route handlers.


.. tip::
    We still only scratched the surface of different patterns for authorizing
    data access, and the kinds of policies you can write with oso.

    For more on access control patters, continue to :doc:`patterns`.

    For more on writing authorization policies, head over to :doc:`/using/policies/index`.

.. toctree::
    :hidden:

    Guide <self>
    patterns
