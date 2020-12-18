=====================
Add roles to your app
=====================


.. TODO: intro

1. Set up the application
=========================

Install the oso SQLAlchemy package
----------------------------------

Install the ``oso_sqlalchemy`` package and import it in your code. Alternatively, if you are
starting from scratch and need a requirements.txt file, clone the sample application `here <TODO>`_

.. code-block:: shell

    pip install sqlalchemy_oso


Add a method to intialize oso and make the oso instance available to your application code.

.. code-block:: python
    :caption: :fab:`python` __init__.py

    from oso import Oso
    from sqlalchemy_oso import register_models, set_get_session

    base_oso = Oso()

    def init_oso(app):
        register_models(base_oso, Base)
        set_get_session(base_oso, lambda: g.session)
        base_oso.load_file("app/authorization.polar")
        app.oso = base_oso



Create a users model
---------------------

Add a user model that will represent your app's users (if you don't already have one).

.. TODO: make this a literal include

.. code-block:: python
    :caption: :fab:`python` models.py

    from sqlalchemy.types import Integer, String
    from sqlalchemy.schema import Column, ForeignKey
    from sqlalchemy.orm import relationship
    from sqlalchemy.ext.declarative import declarative_base


    Base = declarative_base()

    class User(Base):
        __tablename__ = "users"

        id = Column(Integer, primary_key=True)
        email = Column(String())

        def repr(self):
            return {"id": self.id, "email": self.email}



Create an organizations model
------------------------------

Add an organization model that will represent the organizations or tenants
that users belong to. The roles you create will be scoped to this model. If
your app isn't multi-tenant, you can create an ``Application`` model instead,
with one instance used globally.


.. code-block:: python
    :caption: :fab:`python` models.py

    class Organization(Base):
        __tablename__ = "organizations"

        id = Column(Integer, primary_key=True)
        name = Column(String())
        billing_address = Column(String())

        def repr(self):
            return {"id": self.id, "name": self.name}



Add an endpoint that needs authorization
----------------------------------------

Create or choose an existing endpoint that will need authorization to access.
In our sample app we've created 2 endpoints that have different authorization
requirements: one to view repositories and another to view billing
information.

.. code-block:: python
    :caption: :fab:`python` routes.py

    from flask import Blueprint, g, request, current_app
    from .models import User, Organization, Repository

    bp = Blueprint("routes", __name__)


    @bp.route("/orgs/<int:org_id>/repos", methods=["GET"])
    def repos_index(org_id):
        org = g.session.query(Organization).filter(Organization.id == org_id).first()
        repos = g.session.query(Repository).filter_by(organization=org)
        return {f"repos": [repo.repr() for repo in repos]}


    @bp.route("/orgs/<int:org_id>/billing", methods=["GET"])
    def billing_show(org_id):
        org = g.session.query(Organization).filter(Organization.id == org_id).first()
        return {f"billing_address": org.billing_address}



2. Add roles
============

Create the OrganizationRole class using the role mixin
------------------------------------------------------

The oso SQLAlchemy library provides a method to generate a mixin which
creates a role model. Create the mixin by passing in the base, user, and
organization models, as well as the role names. Then create a role model that
extends it.


.. code-block:: python
    :caption: :fab:`python` routes.py

    from sqlalchemy_oso.roles import resource_role_class


    OrganizationRoleMixin = resource_role_class(
        Base, User, Organization, ["OWNER", "MEMBER", "BILLING"]
    )


    class OrganizationRole(Base, OrganizationRoleMixin):
        def repr(self):
            return {"id": self.id, "name": str(self.name)}



Assign role permissions
-----------------------

To give the roles permissions, write an oso policy.
First, call :py:func:`sqlalchemy_oso.roles.enable_roles` to load the base policy for roles.


.. code-block:: python
    :caption: :fab:`python` __init__.py
    :emphasize-lines: 3,12

    from oso import Oso
    from sqlalchemy_oso import register_models, set_get_session
    from sqlalchemy_oso.roles import enable_roles

    base_oso = Oso()

    def init_oso(app):
        register_models(base_oso, Base)
        set_get_session(base_oso, lambda: g.session)
        base_oso.load_file("app/authorization.polar")
        app.oso = base_oso
        enable_roles(base_oso)

You can then write Polar ``role_allow`` rules over ``OrganizationRoles``.

.. code-block:: polar
    :caption: :fa:`oso` authorization.polar

    # ROLE-PERMISSION RELATIONSHIPS

    ## Organization Permissions

    ### All organization roles let users read the organization
    role_allow(role: OrganizationRole, "READ", org: Organization);

    ### Org members can list repos in the org
    role_allow(role: OrganizationRole{name: "MEMBER"}, "LIST_REPOS", organization: Organization);

    ### The billing role can view billing info
    role_allow(role: OrganizationRole{name: "BILLING"}, "READ_BILLING", organization: Organization);

You can also specify a :ref:`hierarchical role ordering <role-hierarchies>` with ``organization_role_order`` rules.

.. code-block:: polar
    :caption: :fa:`oso` authorization.polar

    # ROLE-ROLE RELATIONSHIPS

    ## Role Hierarchies

    ### Specify organization role order (most senior on left)
    organization_role_order(["OWNER", "MEMBER"]);
    organization_role_order(["OWNER", "BILLING"]);

For more details on the roles base policy, see :doc:`/getting-started/builtin-roles/index`.


Enforce the policy
------------------

Add policy checks to your code to control access to the protected endpoints.

.. code-block:: python
    :caption: :fab:`python` routes.py
    :emphasize-lines: 1, 6, 15

    from flask import current_app

    @bp.route("/orgs/<int:org_id>/repos", methods=["GET"])
    def repos_index(org_id):
        org = g.session.query(Organization).filter(Organization.id == org_id).first()
        current_app.oso.authorize(org, actor=g.current_user, action="LIST_REPOS")

        repos = g.session.query(Repository).filter(Repository.organization.has(id=org_id))
        return {f"repos": [repo.repr() for repo in repos]}


    @bp.route("/orgs/<int:org_id>/billing", methods=["GET"])
    def billing_show(org_id):
        org = g.session.query(Organization).filter(Organization.id == org_id).first()
        current_app.oso.authorize(org, actor=g.current_user, action="READ_BILLING")
        return {f"billing_address": org.billing_address}

Our example uses :py:func:`flask_oso.FlaskOso.authorize` to complete the
policy check. If you're not using Flask, check out our general-purpose
:doc:`Python package </using/libraries/python/index>`.


Create an endpoint for assigning roles
--------------------------------------

Until you assign users to roles, they'll receive a ``403 FORBIDDEN`` response if they try to
access either protected endpoint.

Add a new endpoint to your application that users can hit to assign roles.

Call the oso role API
---------------------

Users can be added to a role using `oso.add_to_role`

Configure permissions for role assignments
------------------------------------------

Update the oso policy to specify who is allowed to assign roles.

3. Test it works
================

Run the application
-------------------

Start your server ...

Try it out
----------

Make an API request to ...

As admin, you can assign a user to a role

As a user in a role, you can see X but not Y
