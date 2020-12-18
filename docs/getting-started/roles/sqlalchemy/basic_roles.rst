=====================
Add roles to your app
=====================


.. TODO: intro

1. Set up the application
=========================

Install the oso SQLAlchemy package
-------------------------------------

Install the package and import it in your code. Alternatively, if you are
starting from scratch and need a requirements.txt file, clone the sample application `here <TODO>`_

.. code-block:: shell

    pip install sqlalchemy_oso

Create a users model
---------------------

Add a user model that will represent your app's users (if you don't already have one).

Create an organizations model
------------------------------

Add an organization model that will represent the organizations or tenants that users belong to.

Add an endpoint that needs authorization
----------------------------------------

Create an endpoint that will need authorization to access.

2. Add roles
============

Create the OrganizationRole class using the role mixin
------------------------------------------------------

The oso SQLAlchemy library provides a mixin which creates a role model for users. Create a role model that extends it.

Assign role permissions
-----------------------

Write a policy

Create an endpoint for assigning roles
--------------------------------------

Add a new endpoint to your application that users can hit to assign roles

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
