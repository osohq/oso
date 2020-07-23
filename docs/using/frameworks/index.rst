======================
Framework Integrations
======================

**Coming soon**

oso integrates directly with language specific web frameworks and ORMs to
streamline the process of adding oso to your application.

.. Totally made up code snippet!

.. code-block:: python

    from oso.flask import oso
    from .app import app

    @app.route("/secret")
    @oso.authorize
    def secret_route():
        return "Hello world"

In the meantime...

- Vote & track your favorite framework integration at our `GitHub repository`_.
- Checkout our `blog posts`_ on using oso in your app.
- `Clone`_ an example app in your preferred language

.. todo:: Add link to GitHub issues or other for feedback on which framework
   integrations

.. _GitHub repository: <TODO>
.. _blog posts: <TODO>
.. _Clone: <TODO>

.. todo:: Add link to blog posts about using oso with a framework.
