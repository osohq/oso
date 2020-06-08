========
Context
========

.. role:: polar(code)
   :language: prolog
.. role:: python(code)
   :language: python

Allow rules take in an :ref:`actor <actors>` (which comes from authorization logic) and a :ref:`resource <resources>` (which comes from mapping).
Sometimes you need some additional context information about the environment to write rules over.

Context
-----------

For example, let's say you have a policy like this:

.. literalinclude:: /examples/context/01-context.polar
   :language: polar
   :lines: 6-7

Here we have a very simple allow rule that allows an actor to access any resource if they are an admin.
Maybe we want to also let any actor access any resource when the app is in development mode.
A typical way to flag that an app is running in development or production mode would be to set an environment variable, e.g. :python:`ENV=development` or :python:`ENV=production`.

How would we read that environment variable from polar though? We can use a custom application class that we expose to polar that lets us directly access the environment variables.

.. literalinclude:: /examples/context/02-context.py
   :language: python
   :lines: 1-7

The above class exposes a `var` method that reads the application's environment variables and returns the value asked for.
We can then register the class with :python:`register_python_class`, which will let us use it in polar rules.

We can add a new allow rule that allows an actor to access a resource if the application is in development mode.

.. literalinclude:: /examples/context/01-context.polar
   :language: polar
   :lines: 7-8

Summary
-------

Application classes make it easy to expose any sort of application data to your polar queries. This simple pattern lets you expose
any kind of data you want to use in polar queries, not just :polar:`Actor` and :polar:`Resource` classes.
