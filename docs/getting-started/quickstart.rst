==========
Quickstart
==========

oso helps developers build authorization into their applications. If you've
never used oso before and want to see it in action, this guide is for you.
We're going to walk through how to use oso to add authorization to a simple web
server. You'll be able to follow along by editing the sample application in the
embedded IDE below, or you can view `the complete source code on GitHub
<https://github.com/osohq/oso-python-quickstart>`_.

.. raw:: html

  <iframe height="600px" width="100%"
  src="https://repl.it/@osoHQ/oso-python-quickstart?lite=true" scrolling="no"
  frameborder="no" allowtransparency="true" allowfullscreen="true"
  sandbox="allow-forms allow-pointer-lock allow-popups allow-same-origin
  allow-scripts allow-modals"></iframe>

.. tabs::

    .. tab:: Intro

        Our sample application serves data about expenses submitted by users.
        If you navigate to ``quickstart/expense.py``, you'll see a simple
        ``Expense`` class and some sample data stored in a dictionary.

        The HTTP server code lives in ``quickstart/server.py``, where we have
        defined a route handler for ``GET`` requests to the path
        ``/expenses/:id``. We've already added an authorization check using
        the :doc:`oso python library </using/python/index>` to control access
        to expense resources. You can learn more about how to add oso to your
        application :doc:`here </getting-started/application/index>`.

        .. admonition:: Try it!
            :class: danger try-it

            In the embedded IDE, click the green "Run" button to start the
            server. When the server starts up, it will print out the URL it can
            be reached at. Open a local terminal and make a request using cURL:

            .. code-block:: console
                :class: copybutton

                $ curl <YOUR URL HERE>.repl.co/expenses/1
                Not Authorized!

        You'll get a "Not Authorized!" response because we haven't added any
        rules to our oso policy (in ``expenses.polar``), and oso is
        deny-by-default.

        Let's start implementing our access control scheme by adding some rules
        to the oso policy.

    .. tab:: Adding our first rule

        oso rules are written in a declarative policy language called Polar.
        You can include any kind of rule in a policy, but the oso library is
        designed to evaluate :ref:`allow rules <allow-rules>`, which specify
        the conditions that allow an **actor** to perform an **action** on a
        **resource**.

        .. admonition:: Edit it!
            :class: note

            In our policy file (``expenses.polar``), let's add a rule that
            allows anyone with an email ending in ``"@example.com"`` to view
            all expenses:

            .. literalinclude:: /examples/quickstart/polar/expenses-03-py.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

        .. |str_endswith| replace:: the ``str.endswith`` method
        .. _str_endswith: https://docs.python.org/3/library/stdtypes.html#str.endswith

        The ``Expense`` and ``String`` terms following the colons in the head
        of the rule are :ref:`specializers <Specialization>`, patterns that
        control rule execution based on whether they match the supplied
        argument. This syntax ensures that the rule will only be evaluated when
        the actor is a string and the resource is an instance of the
        ``Expense`` class.

        The second thing to note is that the call to ``.endswith`` is actually
        calling out to |str_endswith|_ defined in the Python standard library.
        The **actor** value passed to oso is a Python string, and oso allows us
        to call methods from Python's standard library on it.

        .. admonition:: Try it!
            :class: danger try-it

            Once we've added our new rule and restarted the web server, every
            user with an ``@example.com`` email should be allowed to view any
            expense:

            .. code-block:: console
                :class: copybutton

                $ curl -H "user: alice@example.com" <YOUR URL HERE>.repl.co/expenses/1
                Expense(...)

        Okay, so what just happened?

        When we ask oso for a policy decision via ``Oso.is_allowed()``, the oso
        engine searches through its knowledge base to determine whether the
        provided **actor**, **action**, and **resource** satisfy any **allow**
        rules. In the above case, we passed in ``"alice@example.com"`` as the
        **actor**, ``"GET"`` as the **action**, and the ``Expense`` object with
        ``id=1`` as the **resource**. Since ``"alice@example.com"`` ends with
        ``@example.com``, our rule is satisfied, and Alice is allowed to view
        the requested expense.

        .. admonition:: Try it!
            :class: danger try-it

            If a user's email doesn't end in ``"@example.com"``, the rule
            fails, and they are denied access:

            .. code-block:: console
                :class: copybutton

                $ curl -H "user: alice@foo.com" <YOUR URL HERE>.repl.co/expenses/1
                Not Authorized!

        If you aren't seeing the same thing, make sure you created your policy
        correctly in ``expenses.polar``.

    .. tab:: Using application data

        We now have some basic access control in place, but we can do better.
        Currently, anyone with an email ending in ``@example.com`` can see all
        expenses -- including expenses submitted by others.

        .. admonition:: Edit it!
            :class: note

            Let's modify our existing rule such that users can only see their
            own expenses:

            .. literalinclude:: /examples/quickstart/polar/expenses-04.polar
                :language: polar
                :caption: :fa:`oso` expenses.polar
                :class: copybutton

        Behind the scenes, oso looks up the ``submitted_by`` field on the
        provided ``Expense`` instance and compares that value against the
        provided **actor**. And just like that, an actor can only see an
        expense if they submitted it!

        .. admonition:: Try it!
            :class: danger try-it

            Alice can see her own expenses but not Bhavik's:

            .. code-block:: console
                :class: copybutton

                $ curl -H "user: alice@example.com" <YOUR URL HERE>.repl.co/expenses/1
                Expense(...)
                $ curl -H "user: alice@example.com" <YOUR URL HERE>.repl.co/expenses/3
                Not Authorized!

            And vice-versa:

            .. code-block:: console
                :class: copybutton

                $ curl -H "user: bhavik@example.com" <YOUR URL HERE>.repl.co/expenses/1
                Not Authorized!
                $ curl -H "user: bhavik@example.com" <YOUR URL HERE>.repl.co/expenses/3
                Expense(...)

        Feel free to play around with the current policy and experiment with
        adding your own rules!

        For example, if you have ``Expense`` and ``User`` classes defined in
        your application, you could write a policy rule in oso that says a
        ``User`` may ``"approve"`` an ``Expense`` if they manage the ``User``
        who submitted the expense and the expense's amount is less than
        $100.00:

        .. code-block:: polar
            :class: copybutton no-select

            allow(approver: User, "approve", expense: Expense) if
                approver = expense.submitted_by.manager
                and expense.amount < 10000;

        In the process of evaluating that rule, the oso engine would call back
        into the application in order to make determinations that rely on
        application data, such as:

        - Which user submitted the expense in question?
        - Who is their manager?
        - Is their manager the user who's attempting to approve the expense?
        - Does the expense's ``amount`` field contain a value less than
          $100.00?

        For more on leveraging application data in an oso policy, check out
        :doc:`/getting-started/policies/application-types`.

.. admonition:: What's next
    :class: tip whats-next

    * Explore how to :doc:`/getting-started/application/index`.
    * Dig deeper on :doc:`/getting-started/policies/index`.
    * Check out oso in action: :doc:`/using/examples/index`.
    * Explore the :doc:`/more/design-principles` behind oso.

------------------------

.. include:: /newsletter.rst
