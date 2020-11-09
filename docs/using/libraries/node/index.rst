.. meta::
  :description: Learn how to use oso with Node.JS to add authorization to your application.

=============================
Node.js Authorization Library
=============================

oso is packaged as a :doc:`npm module </download>` for use in Node.js
applications.

API documentation lives :doc:`here</js/node/index>`.

.. toctree::
    :hidden:

    /js/node/index

To install, see :doc:`installation instructions </download>`.

.. todo:: typescript

Working with JavaScript Types
=============================

oso's Node.js authorization library allows you to write policy rules over
JavaScript types directly. This document explains how different types of
JavaScript values can be used in oso policies.

.. note::
  More detailed examples of working with application objects can be found in :doc:`/using/examples/index`.

Objects
^^^^^^^

You can pass any JavaScript object into oso and access its properties from
your policy (see :ref:`application-types`).

Class Instances
^^^^^^^^^^^^^^^

Any ``new``-able JavaScript object (including ES6-style classes) can be
constructed from inside an oso policy using the :ref:`operator-new` operator if
the constructor (a ``class`` or ``function`` that responds to `the new operator
<https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new>`_)
has been **registered** using the ``#registerClass`` method. An example of
this can be found :ref:`here <application-types>`.

Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^

Polar supports integer and floating point real numbers, as well as booleans (see :ref:`basic-types`).

Strings
^^^^^^^

JavaScript strings are mapped to Polar :ref:`strings`. JavaScript's string methods may be called in policies:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.username.endsWith("example.com");

.. code-block:: javascript
  :caption: :fab:`node-js` app.js

  class User {
    constructor(username) {
      this.username = username;
    }
  }

  const user = new User('alice@example.com');
  oso.isAllowed(user, 'foo', 'bar').then(
    result => assert(result)
  );

.. warning::
  Polar does not support methods that mutate strings in place.

Lists
^^^^^

JavaScript `Arrays <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array>`_
are mapped to Polar :ref:`Lists <lists>`. JavaScript's Array methods may be called in policies:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.groups.includes("HR");

.. code-block:: javascript
  :caption: :fab:`node-js` app.js

  class User {
    constructor(groups) {
      this.groups = groups;
    }
  }

  const user = new User(["HR", "payroll"]);
  oso.isAllowed(user, 'foo', 'bar').then(
    result => assert(result)
  );

.. warning::
  Polar does not support methods that mutate lists in place, unless the list is also returned from the method.

Likewise, lists constructed in Polar may be passed into JavaScript methods:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if actor.hasGroups(["HR", "payroll"]);

.. code-block:: javascript
  :caption: :fas:`node-js` app.js

  class User {
    constructor(groups) {
      this.groups = groups;
    }

    hasGroups(other) {
      return other.every(group => this.groups.includes(group));
    }
  }

  const user = new User(["HR", "payroll"]);
  oso.isAllowed(user, 'foo', 'bar').then(
    result => assert(result)
  );

There is currently no syntax for random access to a list element within a policy;
i.e., there is no Polar equivalent of the JavaScript expression ``user.groups[1]``.
To access the elements of a list, you may iterate over it with :ref:`operator-in`
or destructure it with :ref:`pattern matching <patterns-and-matching>`.

.. todo:: Mention no dictionary type conversion?

Iterators
^^^^^^^^^

You may iterate over a (synchronous or `asynchronous
<https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/asyncIterator>`_) JavaScript
`iterator <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols>`_
using the Polar :ref:`operator-in` operator:

.. code-block:: polar
  :caption: :fa:`oso` policy.polar

  allow(actor, action, resource) if "payroll" in actor.getGroups();

.. code-block:: javascript
  :caption: :fab:`node-js` app.js

  class User {
    getGroups() {
      return ["HR", "payroll"].values();
    }
  }

  const user = new User();
  oso.isAllowed(user, 'foo', 'bar').then(
    result => assert(result)
  );


Promises
^^^^^^^^

oso will ``await`` any `Promise
<https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise>`_
and then use the resolved value during evaluation of a policy.

Summary
^^^^^^^

.. list-table:: JavaScript → Polar Types Summary
  :width: 500 px
  :header-rows: 1

  * - JavaScript type
    - Polar type
  * - number (Integer)
    - Integer
  * - number (Float)
    - Float
  * - boolean
    - Boolean
  * - Array
    - List
  * - string
    - String
