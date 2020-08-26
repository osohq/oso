====================
Multiple Actor Types
====================

Recall that in oso, :ref:`actors` represent request-makers, the "who" of an authorization request.
Actors are commonly human users, but might also be machines, servers, or other applications.
Many applications support multiple types of Actors, and often different Actor types require different
authorization logic.

In this guide, we'll walk through a policy for an application with two Actor types: **Customers** and
**Internal Users**.

.. note:: This guide assumes you are familiar with oso's :doc:`/more/glossary`.


A Tale of Two Actors
=====================

Our example application has customers and internal users. Customers are allowed to access the customer dashboard,
and internal users are allowed to access the customer dashboard as well as an internal dashboard. We can write a simple
policy to express this logic.

.. tabs::
    .. group-tab:: Python

        Let's start by defining Python classes to represent customers and internal users:

        .. literalinclude:: /examples/user_types/python/01-user_classes.py
            :caption: :fab:`python` user_types.py
            :language: python
            :start-after: classes-start
            :end-before: classes-end

    .. group-tab:: Ruby

        Let's start by defining Ruby classes to represent customers and internal users:

        .. literalinclude:: /examples/user_types/ruby/01-user_classes.rb
            :caption: :fas:`gem` user_types.rb
            :language: ruby
            :start-after: classes-start
            :end-before: classes-end

    .. group-tab:: Java

        Java example coming soon.

    .. group-tab:: Node.js

        Node.js example coming soon.

Note that if we already had classes in our application that represented customers and internal users,
we could have simply decorated them with :py:func:`oso.polar_class`.

We can now write a simple policy over these Actor types:

.. literalinclude:: /examples/user_types/user_policy.polar
    :caption: :fa:`oso` user_types.polar
    :language: polar
    :start-after: simple-start
    :end-before: simple-end

This policy uses :ref:`specialized rules <specializer>` to control rules execution based on
the Actor types that is passed into the authorization request.


To finish securing our dashboards, we need to **enforce** our policy by
adding authorization requests to our application.
Where and how authorization requests are used is up to the application developer.


For our example, making a request might look like this:

.. tabs::
    .. group-tab:: Python

        .. literalinclude:: /examples/user_types/python/01-user_classes.py
            :caption: :fab:`python` user_types.py
            :start-after: app-start
            :end-before: app-end

    .. group-tab:: Ruby

        .. literalinclude:: /examples/user_types/ruby/01-user_classes.rb
            :caption: :fas:`gem` user_types.rb
            :language: ruby
            :start-after: app-start
            :end-before: app-end

    .. group-tab:: Java

        Java example coming soon.

    .. group-tab:: Node.js

        Node.js example coming soon.

Hooray, our customer and internal dashboards are now secure!

Adding Actor Attributes
=======================

Since we saved so much time on authorization, we've decided to add another dashboard to our application,
an **accounts dashboard**. The accounts dashboard should only be accessed by **account managers** (a type of internal user).
Since we're experts at securing dashboards, we should be able to add this authorization logic to our policy in no time.
A simple way to solve this problem is with RBAC.


We can add a ``role`` attribute to our ``InternalUser`` class:

.. tabs::
    .. group-tab:: Python

        .. literalinclude:: /examples/user_types/python/02-user_classes.py
            :caption: :fab:`python` user_types.py
            :start-after: internal-start
            :end-before: internal-end

    .. group-tab:: Ruby

        .. literalinclude:: /examples/user_types/ruby/02-user_classes.rb
            :caption: :fas:`gem` user_types.rb
            :language: ruby
            :start-after: internal-start
            :end-before: internal-end

    .. group-tab:: Java

        Java example coming soon.

    .. group-tab:: Node.js

        Node.js example coming soon.


Then add the following rule to our policy:

.. literalinclude:: /examples/user_types/user_policy.polar
    :caption: :fa:`oso` user_types.polar
    :language: polar
    :start-after: rbac-start
    :end-before: rbac-end

This example shows a clear benefit of using different classes to represent different Actor types: the ability
to add custom attributes. We can add attributes specific to internal users, like roles, to the ``InternalUser`` class
without adding them to all application users.

We've been able to secure the accounts dashboard with a few lines of code, but we're not done yet!

Account managers are also allowed to access **account data**, but only for accounts that they manage.
In order to implement this logic, we need to know the accounts of each account manager.

This is a compelling case for creating a new Actor type for account managers that has its own
attributes:

.. tabs::
    .. group-tab:: Python

        .. literalinclude:: /examples/user_types/python/02-user_classes.py
            :caption: :fab:`python` user_types.py
            :start-after: account-start
            :end-before: account-end

    .. group-tab:: Ruby

        .. literalinclude:: /examples/user_types/ruby/02-user_classes.rb
            :caption: :fas:`gem` user_types.rb
            :language: ruby
            :start-after: account-start
            :end-before: account-end

    .. group-tab:: Java

        Java example coming soon.

    .. group-tab:: Node.js

        Node.js example coming soon.

Since account managers are also internal users, we've made the ``AccountManager`` type extend ``InternalUser``.
This means that our rules that specialize on ``InternalUser`` will still execute for account managers (see :doc:`inheritance`).

Let's add the following lines to our policy:

.. literalinclude:: /examples/user_types/user_policy.polar
    :caption: :fa:`oso` user_types.polar
    :language: polar
    :start-after: manager-start
    :end-before: manager-end



The first rule replaces the RBAC rule we previously used to control access to the accounts dashboard.
The second rule controls access to account data. For the purposes of this example, let's assume that ``AccountData`` is a resource that has an ``account_id``
attribute.

We can update our application code slightly to generate ``AccountManager`` users:

.. tabs::
    .. group-tab:: Python

        .. literalinclude:: /examples/user_types/python/02-user_classes.py
            :caption: :fab:`python` user_types.py
            :start-after: account-end
            :emphasize-lines: 5-6

    .. group-tab:: Ruby

        .. literalinclude:: /examples/user_types/ruby/02-user_classes.rb
            :caption: :fas:`gem` user_types.rb
            :language: ruby
            :start-after: account-end
            :emphasize-lines: 5-7

    .. group-tab:: Java

        Java example coming soon.

    .. group-tab:: Node.js

        Node.js example coming soon.

We've now successfully secured all three dashboards and customer account data.

Summary
=======

It is common to require different authorization logic for different types of application users. In this example,
we showed how to use different Actor types to represent different users in oso. We wrote policies with rules
that specialized on the type of Actor, and even added attributes to some actor types that we used in the policy.
We also demonstrated how inheritance can be used to match rules to multiple types of Actors.

.. admonition:: What's next
    :class: tip whats-next

    * :doc:`Download oso </download>` to apply this
      technique in your app.
    * Check out other :doc:`index`.
