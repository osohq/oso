=====================
Roles with SQLAlchemy
=====================

The :py:data:`sqlalchemy_oso.roles` module provides out-of-the-box Role-Based Access Control features that
let you create a roles system  with a few lines of code, and specify role permissions
in a declarative oso policy.

This guide walks you through how to use ``sqlalchemy_oso`` to add basic roles to a multi-tenant app.

    Note: we're using a Flask app for this example, but the
    ``sqlalchemy_oso`` library can be used with any Python application.

1. Set up the application
=========================

Install the oso SQLAlchemy package
----------------------------------

Install the ``sqlalchemy_oso`` package and import it in your code.
Alternatively, if you are starting from scratch and need a
``requirements.txt`` file, clone the `sample application
<https://github.com/osohq/oso-sqlalchemy-roles-guide/tree/main>`_.

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ pip install sqlalchemy_oso


Add a method to initialize oso and make the oso instance available to your application code.
Call :py:func:`sqlalchemy_oso.roles.enable_roles` to load the base oso policy for roles.

.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/__init__.py
    :class: copybutton
    :language: python
    :lines: 11-14, 57-66
    :caption: :fab:`python` __init__.py




Create a users model
---------------------

Add a user model that will represent your app's users (if you don't already have one).


.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/models.py
    :class: copybutton
    :language: python
    :lines: 9-10, 14-21
    :caption: :fab:`python` models.py


Create an organizations model
------------------------------

Add an organization model that will represent the organizations or tenants
that users belong to. The roles you create will be scoped to this model. If
your app isn't multi-tenant, you can create an ``Application`` model instead,
with one instance used globally.


.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/models.py
    :class: copybutton
    :language: python
    :lines: 24-32
    :caption: :fab:`python` models.py



Add an endpoint that needs authorization
----------------------------------------

Create or choose an existing endpoint that will need authorization to access.
In our sample app we've created 2 endpoints that have different authorization
requirements: one to view repositories and another to view billing
information.

Add policy checks to your code to control access to the protected endpoints.


.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/routes.py
    :class: copybutton
    :language: python
    :lines: 1-2, 5-6, 13-27
    :caption: :fab:`python` routes.py

Our example uses :py:func:`flask_oso.FlaskOso.authorize` to complete the
policy check. If you're not using Flask, check out our general-purpose
:doc:`Python package </using/libraries/python/index>`.

2. Add roles
============

Create the OrganizationRole class using the role mixin
------------------------------------------------------

The oso SQLAlchemy library provides a method to generate a mixin which
creates a role model. Create the mixin by passing in the base, user, and
organization models, as well as the role names. Then create a role model that
extends it.


.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/models.py
    :class: copybutton
    :language: python
    :lines: 6, 50-58
    :caption: :fab:`python` models.py



Specify role permissions
------------------------

To give the roles permissions, write an oso policy.

Since we already called :py:func:`sqlalchemy_oso.roles.enable_roles` in our ``init_oso()`` method,
you can write Polar ``role_allow`` rules over ``OrganizationRoles``.

.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/authorization.polar
    :class: copybutton
    :language: polar
    :lines: 7-14
    :caption: :fa:`oso` authorization.polar


You can also specify a :ref:`hierarchical role ordering <role-hierarchies>` with ``organization_role_order`` rules.

.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/authorization.polar
    :class: copybutton
    :language: polar
    :lines: 24-26
    :caption: :fa:`oso` authorization.polar


For more details on the roles base policy, see :doc:`/getting-started/roles/index`.


Create an endpoint for assigning roles
--------------------------------------

Until you assign users to roles, they'll receive a ``403 FORBIDDEN`` response if they try to
access either protected endpoint.

Add a new endpoint to your application that users can hit to assign roles. To
control who can assign roles, add another call to
:py:func:`flask_oso.FlaskOso.authorize`.

Use the oso role API to create role assignments with
:py:meth:`sqlalchemy_oso.roles.add_user_role` and
:py:meth:`sqlalchemy_oso.roles.reassign_user_role`.

.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/routes.py
    :class: copybutton
    :language: python
    :lines: 30-44
    :emphasize-lines: 4,11,13
    :caption: :fab:`python` routes.py


Configure permissions for role assignments
------------------------------------------

Update the oso policy to specify who is allowed to assign roles.

.. literalinclude:: /examples/roles/sqlalchemy/oso-sqlalchemy-roles-guide/app/authorization.polar
    :class: copybutton
    :language: polar
    :lines: 16-17
    :caption: :fa:`oso` authorization.polar


3. Test it works
================

Run the application
-------------------

Start the server.

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ flask run
    * Running on http://127.0.0.1:5000/ (Press CTRL+C to quit)

Make a simple request.

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ curl --header "user: ringo@beatles.com" localhost:5000/
    Hello ringo@beatles.com


Try it out
----------

Try to access the protected endpoints. Access should be granted or denied
based on your policy. Our sample app includes some fixture data for testing.
To run the server with fixture data, set the ``FLASK_APP`` environment
variable.

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ export FLASK_APP="app:create_app(None, True)"
    $ flask run
    * Running on http://127.0.0.1:5000/ (Press CTRL+C to quit)

Our policy says that users with the
"OWNER" role can assign roles, users with the ``"MEMBER"`` role can view
repositories, and users with the ``"BILLING"`` role can view billing info. Also, the
``"OWNER"`` roles inherits the permissions of the ``"MEMBER"`` and "BILLING" roles.

Paul is a member of "The Beatles" organization, so he can view repositories but not
billing info:

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ curl --header "user: paul@beatles.com" localhost:5000/orgs/1/repos
    {"repos":[{"id":1,"name":"Abbey Road"}]}

    $ curl --header "user: paul@beatles.com" localhost:5000/orgs/1/billing
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Unauthorized</p>

John is the owner of "The Beatles" so he can assign roles:

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ curl --header "Content-Type: application/json" \
    --header "user: john@beatles.com"  \
    --request POST \
    --data '{"name":"BILLING", "user_email":"ringo@beatles.com"}' \
    http://localhost:5000/orgs/1/roles
    created a new role for org: 1, ringo@beatles.com, BILLING

But Ringo isn't an owner, so his access should be denied:

.. code-block:: console
    :class: copybutton
    :caption: $_

    $ curl --header "Content-Type: application/json" \
    --header "user: ringo@beatles.com"  \
    --request POST \
    --data '{"name":"BILLING", "user_email":"ringo@beatles.com"}' \
    http://localhost:5000/orgs/1/roles
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
    <title>403 Forbidden</title>
    <h1>Forbidden</h1>
    <p>Unauthorized</p>

The fully-implemented GitHub sample app, complete with tests, can be found `here
<https://github.com/osohq/oso-sqlalchemy-roles-guide/tree/basic_roles_complete>`_.


