.. meta::
   :description: A detailed guide for incorporating oso's policy engine into your application.

=======================
Add To Your Application
=======================

This guide covers a little more detail about how to add oso to your application.

Whereas in the :doc:`/getting-started/quickstart` we zoomed through an example
of authorization in a simple web server, in this guide we'll show some more
practical examples in the context of a more realistic application.

.. tabs::
    .. group-tab:: Python

        Our sample expenses application is built with Flask, but we are not using
        anything from oso that is unique to Flask, and the same patterns we cover here
        can be used anywhere.

        We highly encourage you to follow along with the code by cloning the example repository
        and trying it out. The code can be found here:

        .. todo:: Insert little GitHub snippet box + actually have a real link

        :fab:`github` `osohq/oso-flask-tutorial <https://github.com/osohq/oso-flask-tutorial>`_

    .. group-tab:: Java

        Our sample expenses application is a Maven project built with Spring Boot.
        We are not using anything from oso that is unique to Spring Boot, and the same patterns we cover here
        can be used anywhere.

        We highly encourage you to follow along with the code by cloning the example repository
        and trying it out. The code can be found here:

        .. todo:: Insert little GitHub snippet box + actually have a real link

        :fab:`github` `osohq/oso-spring-tutorial <https://github.com/osohq/oso-spring-tutorial>`_

Our expenses application reads from a sqlite database, and has a few simple endpoints for returning
results. We encourage you to take a look around before continuing!

Running The Example
===================

.. tabs::
    .. group-tab:: Python

        The application has a few requirements, including Flask and, of course, oso.
        We recommend installing these within a virtual environment:

        .. code-block:: console

            $ git clone https://github.com/osohq/oso-flask-tutorial/
            $ cd oso-flask-tutorial/
            $ python3 -m venv venv
            $ source venv/bin/activate
            $ pip3 install -r requirements.txt
            $ FLASK_ENV=development flask run --extra-files app/authorization.polar

        The example comes with some example data, which you can load with:

        .. code-block:: console

            $ sqlite3 expenses.db ".read expenses.sql"

    .. group-tab:: Java

        After cloning the example app, make sure to run ``mvn install`` to download the necessary JARs.

        The example comes with some example data, which you can load by calling:

        .. code-block:: console

            $ sqlite3 expenses.db ".read expenses.sql"

        You can then run the app by calling

        .. code-block:: console

            $ mvn spring-boot:run


In Your Application
===================

.. tabs::
    .. group-tab:: Python

        There are two pieces to get started with oso in your application.
        The policy file, and the ``oso.is_allowed`` call.

        The policy file captures the authorization logic you want to apply in
        your application, and the ``oso.is_allowed`` call is used to
        enforce that policy in your application.

    .. group-tab:: Java

        There are two pieces to get started with oso in your application.
        The policy file, and the ``oso.isAllowed`` call.

        The policy file captures the authorization logic you want to apply in
        your application, and the ``oso.isAllowed`` call is used to
        enforce that policy in your application.

When starting out, it is reasonable to capture all policy logic in
a single ``authorization.polar`` file, as we have done here.
However, over time you will want to break it up into multiple
files.

Additionally, there are two main places where we want to enforce our
authorization logic: at the request/API layer, and at the data access
layer.

The goal of the former is to restrict which *actions* a user can take in
your application, e.g. are they allowed to fetch the
expenses report via the ``GET /expenses/report`` route.

The goal of the latter is to restrict them from viewing data they shouldn't have
access to, e.g. they should not be able to see other users' data.


Add oso
-------

.. tabs::
    .. group-tab:: Python

        In our sample application, we are storing our policies in the ``authorization.polar``
        file, and all of the authorization in the application is managed through the
        ``authorization.py`` file.

        In the application, we need to:

        1. Create the oso instance
        2. Load in policy files.
        3. :doc:`Register application classes </getting-started/policies/application-types>`
        4. Attach the oso instance to the application

        We have achieved this using the ``init_oso`` method:

        .. literalinclude:: /examples/application/expenses-flask/app/authorization.py
            :caption: :fab:`python` authorization.py
            :language: python
            :lines: 25-

    .. group-tab:: Java

        In our sample application, we are storing our policies in the ``authorization.polar``
        file.

        In the application, we need to:

        1. Create the oso instance
        2. Load in policy files.
        3. :doc:`Register application classes </getting-started/policies/application-types>`
        4. Attach the oso instance to the application

        We have achieved this using the ``setupOso`` method, in ``Application.java``.

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Application.java
            :caption: :fab:`java` Application.java
            :language: java
            :lines: 24-34

We can now access this ``oso`` instance anywhere in our application, and specify
which policy files are loaded in the application configuration.

Authorizing Routes
------------------

The first thing we want to add to our application is some simple authorization
to allow some users to only have access to certain routes if they are logged in.

.. tabs::
    .. group-tab:: Python

        We can apply apply authorization to **every** incoming request by setting up
        a middleware function that runs before every request using ``before_app_request``:

        .. literalinclude:: /examples/application/expenses-flask/app/authorization.py
            :caption: :fab:`python` authorization.py
            :language: python
            :lines: 9-14


    .. group-tab:: Java

        We can apply apply authorization to **every** incoming request by setting up
        a request ``Interceptor``, with a ``prehandle`` function that runs before every request:

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Authorizer.java
            :caption: :fab:`java` Authorizer.java
            :language: java
            :lines: 23-36

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


But we also have a ``/whoami`` route that returns a short description
of the current user. We want to make sure only authenticated users can
see this.

We have two different user types here: the ``Guest`` class and the ``User`` class.
The latter corresponds to users who have authenticated.

.. tabs::
    .. group-tab:: Python

        .. literalinclude:: /examples/application/expenses-flask/app/user.py
            :caption: :fab:`python` user.py
            :language: python
            :lines: 52-53

        .. literalinclude:: /examples/application/expenses-flask/app/user.py
            :caption: :fab:`python` user.py
            :language: python
            :lines: 16-25


    .. group-tab:: Java

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Guest.java
            :caption: :fab:`java` Guest.java
            :language: java
            :lines: 3-7

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/User.java
            :caption: :fab:`java` User.java
            :language: java
            :lines: 8-25

We can use :ref:`specializer rules <specializer>` to only allow the request
when the actor is an instance of a ``User``:

.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :language: polar
    :lines: 6-7

.. tabs::
    .. group-tab:: Python

        .. code-block:: console

            $ curl localhost:5000/whoami
            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
            <title>403 Forbidden</title>
            <h1>Forbidden</h1>
            <p>Not Authorized!</p>

            $ curl -H "user: alice@foo.com"  localhost:5000/whoami
            You are alice@foo.com, the CEO at Foo Industries. (User ID: 1)

    .. group-tab:: Java

        .. code-block:: console

            $ curl -i localhost:5000/whoami
            HTTP/1.1 403

            $ curl -H "user: alice@foo.com"  localhost:5000/whoami
            You are alice@foo.com, the CEO at Foo Industries. (User ID: 1)

.. tip::

    Interested in understanding more about what is happening here?
    Check out the :doc:`/using/examples/user_types` example.

.. tabs::
    .. group-tab:: Python

        The inputs to the ``is_allowed`` call are the current user, the HTTP method,
        and the HTTP request. This information can often be enough to cover a large
        number of uses. For example, if we know that some paths should only
        be accessed by certain roles, we can certainly check for this at this point.

    .. group-tab:: Java

        The inputs to the ``isAllowed`` call are the current user, the HTTP method,
        and the HTTP request. This information can often be enough to cover a large
        number of uses. For example, if we know that some paths should only
        be accessed by certain roles, we can certainly check for this at this point.

In a RESTful application, you can also consider "mapping" authorization
logic from the HTTP path to actions and classes in the application.

For example:

.. tabs::
    .. group-tab:: Python

        .. code-block:: polar
            :caption: :fa:`oso` authorization.polar

                allow(user, "GET", http_request) if
                    http_request.startswith("/expenses/")
                    and allow(user, "read", Expense);

    .. group-tab:: Java

        .. code-block:: polar
            :caption: :fa:`oso` authorization.polar

                allow(user, "GET", http_request) if
                    http_request.startsWith("/expenses/")
                    and allow(user, "read", Expense);

This rule is translating something like ``GET /expenses/3`` into a check
whether the user should be allowed to "read" the ``Expense`` class.

However, what if we want to have more fine-grained control? And authorize access
to the precise resource at ``/expenses/3``? We'll cover that in the next
section.

Authorizing Access to Data
--------------------------

.. tabs::
    .. group-tab:: Python

        In the :doc:`/getting-started/quickstart`, our main objective was to
        determine who could "GET" expenses. Our final policy looked like:

        .. literalinclude:: /examples/quickstart/polar/expenses-02-python.polar
            :language: polar
            :caption: :fa:`oso` expenses.polar

        In our expenses sample application, we have something similar,
        but we've rewritten the policy to use a new ``submitted`` predicate in case we want
        to change the logic in the future.

        .. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
            :caption: :fa:`oso` authorization.polar
            :language: polar
            :lines: 21-25


        To handle authorizing access to data, we've implemented a little helper method
        for us to use throughout the application:

        .. literalinclude:: /examples/application/expenses-flask/app/authorization.py
            :caption: :fab:`python` authorization.py
            :language: python
            :lines: 17-22

        ... so authorizing the GET request looks like:

        .. literalinclude:: /examples/application/expenses-flask/app/expense.py
            :caption: :fab:`python` expense.py
            :language: python
            :lines: 50-52

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
            :lines: 30-32

        .. code-block:: console

            $ curl -H "user: alice@foo.com" localhost:5000/organizations/1
            Organization(name='Foo Industries', id=1)

            $ curl -H "user: alice@foo.com" localhost:5000/organizations/2
            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
            <title>403 Forbidden</title>
            <h1>Forbidden</h1>
            <p>Not Authorized!</p>

    .. group-tab:: Java

        In the :doc:`/getting-started/quickstart`, our main objective was to
        determine who could "GET" expenses. Our final policy looked like:

        .. literalinclude:: /examples/quickstart/polar/expenses-02-java.polar
            :language: polar
            :caption: :fa:`oso` expenses.polar

        In our expenses sample application, we have something similar,
        but we've rewritten the policy to use a new ``submitted`` predicate in case we want
        to change the logic in the future.

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/oso/authorization.polar
            :caption: :fa:`oso` authorization.polar
            :language: polar
            :lines: 21-25

        To handle authorizing access to data, we've implemented a little helper method
        for us to use throughout the application:

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Authorizer.java
            :caption: :fab:`java` Authorizer.java
            :language: java
            :lines: 49-58

        ... so authorizing the GET request looks like:

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Controller.java
            :caption: :fab:`java` Controller.java
            :language: java
            :lines: 55-63

        Let's give it a try!

        .. code-block:: console

            $ curl -i localhost:5000/expenses/2
            HTTP/1.1 403

            $ curl -H "user: alice@foo.com" localhost:5000/expenses/2
            Expense(amount=17743, description='Pug irony.', user_id=1, id=2)

        This pattern is pretty convenient. We can easily apply it elsewhere:

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Controller.java
            :caption: :fab:`java` Controller.java
            :language: java
            :lines: 65-73

        .. code-block:: console

            $ curl -H "user: alice@foo.com" localhost:5000/organizations/1
            Organization(name='Foo Industries', id=1)

            $ curl -i -H "user: alice@foo.com" localhost:5000/organizations/2
            HTTP/1.1 403


Applying this pattern to authorizing data means that the objects we are passing
in to the policy evaluation are already fairly rich objects, with attributes and
methods we can use to make policy decisions. When starting out, it might be more
convenient to apply in the route handler itself, but try moving it even closer
to the data access layer. For example, if we moved the ``authorize`` call into
the ``Expense.lookup`` method, then anywhere our application wants to retrieve
an expense, we are assured that the user does indeed have access to it.


Your Turn
=========

We currently have a route with no authorization - the submit endpoint.
We have a rule that allows anyone to PUT to the submit endpoint, but
we want to make sure only authorized expenses are submitted.


.. literalinclude:: /examples/application/expenses-flask/app/authorization.polar
    :caption: :fa:`oso` authorization.polar
    :lines: 18

.. tip::

    The ``allow_by_path`` rule is a custom rule in our policy that operates
    on an actor, action, first url path fragment, and the remaining path
    fragment.  A ``PUT /expenses/submit`` request would try to authorize using
    the ``allow_by_path(actor, "PUT", "expenses", ["submit"])`` rule. See
    `our policy`_ for more detail.

.. _`our policy`: https://github.com/osohq/oso-flask-tutorial/blob/ecc39c601057bcfdb952e35da616fe2e1ea00a22/app/authorization.polar#L10

.. tabs::
    .. group-tab:: Python

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
            :lines: 56-64
            :emphasize-lines: 7-8

        We could change the first highlighted line to:

        .. code-block:: python

            expense = authorize("create", Expense(**expense_data))

        This checks the current user is authorized to create the expense.
        If this passes, then we can happily move on to the ``expense.save()``.

        Now, nobody will be able to submit expenses, since we haven't yet
        added any rules saying they can.

    .. group-tab:: Java

        Right now you can see that anyone can submit an expense:

        .. code-block:: console

            $ curl -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
            Expense(amount=100, description='Gummy Bears', user_id=1, id=108)


        How might we use the ``authorize`` method from before, to make sure that
        we check the user is allowed to ``create`` this expense?
        We would like to do the authorization on the full ``Expense`` object,
        but before it is persisted to the database, so perhaps before this line:

        .. literalinclude:: /examples/application/expenses-spring-boot/src/main/java/com/example/springboot/Controller.java
            :caption: :fab:`java` Controller.java
            :language: java
            :lines: 75-85
            :emphasize-lines: 6

        We could change the highlighted line to:

        .. code-block:: java

            ((Expense) authorizer.authorize("create", expense)).save();


        This checks the current user is authorized to create the expense.
        If this passes, then we can happily move on to the ``expense.save()``.

        Now, nobody will be able to submit expenses, since we haven't yet
        added any rules saying they can.

.. admonition:: Add a new rule

    Try editing ``authorization.polar`` to add a rule saying that
    a user can create an expense for which they are assigned as the
    submitter of the expense.

Once you have it working, you can test it by verifying as follows:

.. tabs::
    .. group-tab:: Python

        .. code-block:: console

            $ curl -H "user: alice@foo.com" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
            Expense(amount=100, description='Gummy Bears', user_id=1, id=111)

            $ curl -H "user: alice@foo.com" -X PUT -d '{"user_id": 2, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
            <title>403 Forbidden</title>
            <h1>Forbidden</h1>
            <p>Not Authorized!</p>

    .. group-tab:: Java

        .. code-block:: console

            $ curl -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"user_id": 1, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
            Expense(amount=100, description='Gummy Bears', user_id=1, id=111)

            $ curl -i -H "user: alice@foo.com" -H "Content-Type: application/json" -X PUT -d '{"user_id": 2, "amount": 100, "description": "Gummy Bears"}' localhost:5000/expenses/submit
            HTTP/1.1 403

Summary
=======

In this guide, we showed a few examples of how to add oso to a more realistic
application. We added some route-level authorization to control who is allowed
to make requests to certain routes. We also used a new ``authorize`` method to
make it convenient to add data access controls to our route handlers.

.. admonition:: What's next
    :class: tip whats-next

    * To explore integrating oso in your app in more depth continue to
      :doc:`/getting-started/application/patterns`.
    * For a deeper introduction to policy syntax see:
      :doc:`/getting-started/policies/index`.
    * For reference on using oso with your language, see
      :doc:`/using/libraries/index`.
    * Clone this example on `GitHub`_ to check it out further.

.. _`GitHub`: https://github.com/osohq/oso-flask-tutorial

.. toctree::
    :hidden:

    Guide <self>
    patterns
