django-oso API Reference
========================

Authorization
-------------

.. autofunction:: django_oso.auth.authorize

.. autofunction:: django_oso.auth.skip_authorization

.. _django-middleware:

Middleware
----------

.. autoclass:: django_oso.middleware.ReloadPolicyMiddleware

.. autoclass:: django_oso.middleware.RequireAuthorization

.. autoclass:: django_oso.middleware.RouteAuthorization

View Decorators
---------------

.. autofunction:: django_oso.decorators.authorize

.. autofunction:: django_oso.decorators.authorize_request

.. autofunction:: django_oso.decorators.skip_authorization

List endpoint authorization
---------------------------

The oso Django integration includes `list filtering
<https://docs.osohq.com/getting-started/list-filtering/index.html>`_ support for Django models.

.. note::

    These features are in preview and will be stabilized in a
    future release. Please `join our Slack`_ to provide feedback or discuss with
    the engineering team.

Usage
+++++

See the `list filtering usage guide
<https://docs.osohq.com/getting-started/list-filtering/django.html>`_ for more information.

API Reference
+++++++++++++

.. autofunction:: django_oso.auth.authorize_model

.. autoclass:: django_oso.models.AuthorizedModel

.. autoclass:: django_oso.models.AuthorizedQuerySet


.. _`join our Slack`: http://join-slack.osohq.com/

Oso
---

.. autodata:: django_oso.oso.Oso
