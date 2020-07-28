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
meantime, **let us know your preferred framework/ORM** in our 
`Github repository <https://github.com/osohq/oso>`_,
and sign up for our newsletter in the footer anywhere on our docs if you'd like
to stay up to speed on the latest product updates.

.. todo:: Add link to blog posts about using oso with a framework.
.. todo:: Add link to example app.
