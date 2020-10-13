======
Django
======

The oso Django integration adopts Django conventions and provides middleware,
view decorators and ORM integrations to make it easier to use oso with Django.

Installation
============

The oso Django integration is available on `PyPI`_ and can be installed using
``pip``::

    $ pip install django-oso

.. _PyPI: https://pypi.org/project/django-oso/

Usage
=====

The ``django_oso`` django plugin contains a reusable django app that makes
authorization with oso and django easy.  To use, ensure ``django_oso`` is in
``INSTALLED_APPS``:

.. code-block:: python
    :caption: :fab:`python` settings.py

    INSTALLED_APPS = [
        'django_oso',
        ...
    ]

Loading policies
----------------

``django_oso`` expects policy files to be included in the ``policy`` directory
of each installed app.  Upon startup, all ``.polar`` files found in that
directory (or sub-directories) will be loaded using
:py:meth:`oso.Oso.load_file`.  To load additional files outside of these
directories, call :py:meth:`~oso.Oso.load_file` on
:py:data:`django_oso.oso.Oso`.

Registering classes & models
----------------------------

Often, authorization rules will be expressed over django models.  Therefore,
``django_oso`` will register every model for each installed app upon startup as
a class with oso. The :py:class:`django.http.HttpRequest` is also registered
under ``HttpRequest``.  Django models are referenced in a Polar file using the
syntax ``app_name::ModelName``. If an app name contains ``.``, for example
``django.contrib.auth``, it will be referenced in oso as
``django::contrib::auth``.

Additional classes can be registered as needed using
:py:meth:`oso.Oso.register_class` on :py:data:`django_oso.oso.Oso`.

Performing authorization
------------------------

To authorize a request, use the :py:func:`django_oso.auth.authorize` function.
It calls
:py:meth:`~oso.Oso.is_allowed`, but provides sensible defaults for working with
Django. The actor defaults to ``request.user``.  The ``action``
defaults to the method of the request.
``resource`` must be provided.

.. tip::

    If you aren't familiar with how oso uses actors, actions, and resources to
    express authorization decisions, see :doc:`/more/glossary` or
    :doc:`/getting-started/quickstart`.

:py:meth:`django_oso.auth.authorize` can be used within route handlers, or in
the data access layer, depending upon how you want to express authorization.

Here's a basic example in a route:


.. code-block:: python
    :emphasize-lines: 7

    def get_expense(request, id):
        try:
            expense = Expense.objects.get(pk=id)
        except Expense.DoesNotExist:
            return HttpResponseNotFound()

        authorize(request, expense, action="read")
        return HttpResponse(expense.json())

Requiring authorization on every request
----------------------------------------

Since :py:func:`~django_oso.auth.authorize` is just a function call, it can be
forgotten.  To enforce authorization on every request, use the
:py:func:`~django_oso.middleware.RequireAuthorization` middleware. Any view that
does not call :py:func:`~django_oso.auth.authorize` or
:py:func:`~django_oso.auth.skip_authorization` will raise an exception.

Route authorization
-------------------

One common usage of :py:func:`django_oso.auth.authorize` is to perform authorization
based on the request object. The
:py:func:`~django_oso.decorators.authorize_request` decorator does this::

    from django_oso.decorators import authorize_request

    @authorize_request
    def auth_route(request):
        pass

Rules can then be written using request
attributes, like the path:

.. code-block:: polar
    :caption: :fa:`oso`

    # Allow any actor to make a GET request to "/".
    allow(_user: User, "GET", http_request: HttpRequest) if
        http_request.path = "/";

To enforce route authorization on all requests (the equivalent of decorating
every route as we did above), use the
:py:meth:`~django_oso.middleware.RouteAuthorization` middleware during
initialization.

Example
=======

Check out the Django integration example app below on GitHub:

:fab:`github` `osohq/oso-django-integration <https://github.com/osohq/oso-django-integration>`_

API Reference
=============

Authorization
-------------

.. autofunction:: django_oso.auth.authorize

.. autofunction:: django_oso.auth.skip_authorization

Middleware
----------

.. autoclass:: django_oso.middleware.RequireAuthorization

.. autoclass:: django_oso.middleware.RouteAuthorization

View Decorators
---------------

.. autofunction:: django_oso.decorators.authorize

.. autofunction:: django_oso.decorators.authorize_request

.. autofunction:: django_oso.decorators.skip_authorization

List endpoint authorization
---------------------------

.. note::

    These features are in preview and will be stabilized in a
    future release. Please `join our Slack`_ to provide feedback or discuss with
    the engineering team.

.. autofunction:: django_oso.auth.authorize_model

.. autoclass:: django_oso.models.AuthorizedModel

.. autoclass:: django_oso.models.AuthorizedQuerySet


.. _`join our Slack`: http://join-slack.osohq.com/

Oso
---

.. autodata:: django_oso.oso.Oso

