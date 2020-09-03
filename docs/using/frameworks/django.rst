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

The ``django_oso`` django plugin contains a resusable django app that makes
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
under ``HttpRequest``.

Additional classes can be registered as needed using
:py:meth:`oso.Oso.register_class` on :py:data:`django_oso.oso.Oso`.

.. warning::

    Currently there are no namespaces for auto registered models.  If
    applications have conflicting module names, an exception will be thrown
    during startup.  This is a known issue, and is tracked in **THIS LINK**.

    .. todo:: Fix this massive limitation.

Performing authorization
------------------------

Requiring authorization on every request
----------------------------------------

Route authorization
-------------------

Example
=======

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

Oso
---

.. autodata:: django_oso.oso.Oso

.. autofunction:: django_oso.oso.reset_oso
