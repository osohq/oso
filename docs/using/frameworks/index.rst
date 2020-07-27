======================
Framework Integrations
======================

Coming soon
===========

oso integrates directly with language-specific web frameworks and ORMs to
streamline the process of adding oso to your application.

.. Totally made up code snippet!

.. code-block:: python

    from oso.flask import oso
    from .app import app

    @app.route("/secret")
    @oso.authorize
    def secret_route():
        return "Hello world"

What do you think?
==================
We are working on updating our documentation with these integrations. In the
meantime, **let us know your preferred framework/ORM** by using our chat on the bottom right
to send us a Slack message.

.. todo:: Add link to blog posts about using oso with a framework.
.. todo:: Add link to example app.
