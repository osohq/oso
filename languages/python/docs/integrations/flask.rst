=================
Flask Integration
=================

HTTP Request Verification
==========================

Currently, Polar is integrated with Flask. The
:py:meth:`oso.Oso.verify_flask_request` can be used to make an authorization
decision for a Flask request.

When called, it will extract an http resource (a built-in resource type) and
query Polar for an allow rule.

HTTP Resource
-------------

The HTTP resource is represented by a named dictionary::

  http{path: "/path/from/request"}

Where the path key is a string representing the request destination.

Action is the lower cased http method name (``get``, ``post``, etc.).

Basic Policy Examples
=====================

The following policy would allow http GET for ``/repository`` to an actor with
username ``dhatch``, and POST for ``/organization`` to an actor with username
``sam``.

::

  allow(actor{user: dhatch}, get, http{path: "/repository/"});
  allow(actor{user: sam}, post, http{path: "/organization/"});
