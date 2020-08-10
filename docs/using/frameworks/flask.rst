Flask
=====

The oso Flask integration provides a more convenient interface to oso for
usage with Flask.

Usage
-----

Initialization
~~~~~~~~~~~~~~

The :py:class:`flask_oso.FlaskOso` class is the entrypoint to the integration.
It must be initialized with the Flask app and oso::

    from flask import Flask
    from oso import Oso

    app = Flask()
    oso = Oso()
    flask_oso = FlaskOso(app=app, oso=oso)

Alternatively, to support the Flask factory pattern, the
:py:meth:`flask_oso.FlaskOso.init_app` method can be used::

    from flask import Flask

    from oso import Oso
    from flask_oso import FlaskOso

    oso = Oso()
    flask_oso = FlaskOso(oso=oso)

    def create_app():
        app = Flask()

        # Initialize oso for this application
        flask_oso.init_app(app)

        return app

    app = create_app()

This factory function can be a useful place for loading policy files, and
calling configuration functions on :py:class:`flask_oso.FlaskOso` like
:py:meth:`flask_oso.FlaskOso.require_authorization`::

    def create_app():
        app = Flask()

        flask_oso.init_app(app)
        flask_oso.require_authorization(app)

        oso.load_file("authorization.polar")
        oso.register_class(Expense)

Performing authorization
~~~~~~~~~~~~~~~~~~~~~~~~

When using the ``flask_oso`` integration, the primary authorization function is
:py:meth:`flask_oso.FlaskOso.authorize`.  It accepts the same arguments as
:py:meth:`oso.Oso.is_allowed`, but provides sensible defaults for working with
Flask. The actor defaults to ``flask.g.current_user`` (this can be
customized, see :py:meth:`flask_oso.FlaskOso.set_get_actor`).  The ``action``
defaults to the method of the current request ``flask.request.method``.
``resource`` must be provided.

.. tip::

    If you aren't familiar with how oso uses actors, actions, and resources to
    express authorization decisions, see :doc:`/more/glossary` or
    :doc:`/getting-started/quickstart`.

:py:meth:`flask_oso.FlaskOso.authorize` can be used within route handlers, or in
the data access layer, depending upon how you want to express authorization.
Here's a basic example in a route::

    @bp.route("/<int:id>", methods=["GET"])
    def get_expense(id):
        expense = Expense.query.get(id)
        if expense is None:
            raise NotFound()

        oso.authorize(action="read", resource=expense)
        return expense.json()

Notice we didn't need to check the return value of ``authorize``.  By default,
a failed authorization will return a ``403 Forbidden`` response for the current
request. This can be controlled with
:py:meth:`flask_oso.FlaskOso.set_unauthorized_action`.

Requiring authorization
~~~~~~~~~~~~~~~~~~~~~~~

The :py:meth:`flask_oso.FlaskOso.authorize` function provides a flexible API for
authorization.  ``flask_oso`` does not dictate where this call should occur.
See :doc:`/getting-started/application/patterns` for more on where oso can be
integrated.

One downside to requiring routes to call :py:meth:`flask_oso.FlaskOso.authorize`
explicitly is that it can potentially be forgotten.  To help detect this, the
:py:meth:`flask_oso.FlaskOso.require_authorization` option can be enabled during
initialization. This will cause an :py:class:`oso.OsoException` to be raised if
a call to :py:meth:`flask_oso.FlaskOso.authorize` **is not** made during the
processing of a request.

Sometimes a route will not need authorization.  To prevent this route from
causing an authorization error, call
:py:meth:`flask_oso.FlaskOso.skip_authorization` during request processing:

.. code-block::
    :emphasize-lines: 18

    oso = Oso()
    flask_oso = FlaskOso()

    def create_app():
        app = Flask()

        flask_oso.init_app(app)
        flask_oso.require_authorization(app)

        oso.load_file("authorization.polar")

        return app

    app = create_app()

    @app.route("/about")
    def about():
        flask_oso.skip_authorization()
        return "about us"

Using decorators
~~~~~~~~~~~~~~~~

Some developers may prefer a decorator-based API for performing authorization.
``flask_oso`` provides two decorators:

:py:func:`flask_oso.skip_authorization` marks a route as not requiring
authorization.  It is the decorator version of
:py:meth:`flask_oso.FlaskOso.skip_authorization`.

:py:func:`flask_oso.authorize` decorates a route and calls
:py:meth:`flask_oso.FlaskOso.authorize` before the route body is entered. For
example::

    from flask_oso import authorize

    @authorize(resource="get_user")
    @app.route("/user")
    def get_user():
        return "current user"

This decorator can be used if the resource is known before entering the request
body.

Route authorization
~~~~~~~~~~~~~~~~~~~

One common usage of :py:func:`flask_oso.authorize` is to perform authorization
based on the Flask request object::

    from flask import request

    @flask_oso.authorize(resource=request)
    @app.route("/")
    def route():
        return "authorized"

A policy can then be written controlling authorization based on request
attributes, like the path:

.. code-block:: polar
    :caption: :fa:`oso`

    # Allow any actor to make a GET request to "/".
    allow(_actor, action: "GET", resource: Request{path: "/"});

To enforce route authorization on all requests (the equivalent of decorating
every route as we did above), use the
:py:meth:`flask_oso.FlaskOso.perform_route_authorization` method during
initialization.

Example
-------

Check out the Flask integration example app below:

.. todo:: github link

API Reference
=============

.. automodule:: flask_oso
    :members:
