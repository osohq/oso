============================
Ruby Authorization Library
============================

oso is packaged as a gem for use in Ruby applications.

API documentation for the gem lives :doc:`here</ruby/index>`.

Working with Ruby Objects
===========================

oso's Ruby authorization library allows you to write policy rules over Ruby objects directly.
This document explains how different types of Ruby objects can be used in oso policies.

.. note::
    More detailed examples of working with application classes can be found in :ref:`auth-models`.

Class Instances
^^^^^^^^^^^^^^^^
You can pass an instance of any Ruby class into oso and access its methods and fields from your policy.

For example:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.is_admin?;

The above rule expects the ``actor`` variable to be a Ruby instance with the attribute ``is_admin?``.
The Ruby instance is passed into oso with a call to ``allow()``:

.. code-block:: ruby
   :caption: app.rb

   class User
      attr_reader :name
      attr_reader :is_admin

      def initialize(name, is_admin)
         @name = name
         @is_admin = is_admin
      end
   end

   user = User.new("alice", true)
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

The code above provides a ``User`` object as the *actor* for our ``allow`` rule. Since ``User`` has an attribute
called ``is_admin``, it is evaluated by the policy and found to be true.

Ruby instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Ruby class has been **registered** using
either the ``#register_class`` method.

Registering classes also makes it possible to use :ref:`specialization` and the
:ref:`operator-matches` with the registered class:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) if actor matches User{name: "alice", is_admin: true};

.. code-block:: ruby
   :caption: app.rb

   OSO.register_class(User)
   user = User.new("alice", true)
   raise "should be allowed" unless oso.allow(user, "foo", "bar")
   raise "should not be allowed" unless not oso.allow(user, "foo", "bar")

Once a class is registered, its class methods can also be called from oso policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor: User, action, resource) if actor.name in User.superusers();

.. code-block:: ruby
   :caption: app.rub

   class User
      # ...
      def self.superusers
         ["alice", "bhavik", "clarice"]
      end
   end

   oso.register_class(User)

   user = User.new("alice", true)
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

Numbers
^^^^^^^
Polar supports both integer and floating point numbers (see :ref:`basic-types`)

Strings
^^^^^^^
Ruby strings are mapped to Polar :ref:`strings`. Ruby's string methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.username.end_with?("example.com");

.. code-block:: ruby
   :caption: app.rb

   class User
      attr_reader :username

      def initialize(username)
         @username = username
      end
   end

   user = User.new("alice@example.com")
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

.. warning::
    Polar does not support methods that mutate strings in place. E.g. `#chomp` will have no effect on
    a string in Polar.

Lists
^^^^^
Ruby `Arrays <https://ruby-doc.org/core-2.7.0/Array.html>`_ are mapped to Polar :ref:`Lists <lists>`. Ruby's Array methods may be accessed from policies:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.groups.include?("HR");

.. code-block:: ruby
   :caption: app.rb

   class User
      attr_reader :groups

      def initialize(groups)
         @groups = groups
      end
   end

   user = User.new(["HR", "payroll"])
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

.. warning::
    Polar does not support methods that mutate lists in place, unless the list is also returned from the method.

Likewise, lists constructed in Polar may be passed into Ruby methods:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.has_groups?(["HR", "payroll"]);

.. code-block:: ruby
   :caption: app.rb

   class User
      def has_groups(groups)
         groups.each {|g|
            if !groups.include? g
               return false
            end
         }
         true
      end
   end

   user = User.new(["HR", "payroll"])
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

Dictionaries
^^^^^^^^^^^^
Ruby dictionaries are mapped to Polar :ref:`dictionaries`:

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.roles.project1 = "admin";

.. code-block:: ruby
   :caption: app.rb

   class User
      attr_reader :roles

      def initialize(roles)
         @roles = roles
      end
   end

   user = User.new({"project1" => "admin"})
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

Likewise, dictionaries constructed in Polar may be passed into Ruby methods.

Enumerators
^^^^^^^^^^^^
Oso handles Ruby `Enumerators <https://ruby-doc.org/core-2.6/Enumerator.html>`_ by evaluating each of the
object's elements one at a time.

.. code-block:: polar
   :caption: policy.polar

   allow(actor, action, resource) if actor.get_group = "payroll";

.. code-block:: ruby
   :caption: app.rb

   class User
      def get_group(self)
         ["HR", "payroll"].to_enum
      end
   end

   user = User.new
   raise "should be allowed" unless oso.allow(user, "foo", "bar")

In the policy above, the right hand side of the `allow` rule will first evaluate ``"HR" = "payroll"``, then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to ``#allow`` will succeed.
Note that if ``#get_group`` returned an array, the rule would fail, as the evaluation would be ``["HR", "payroll"] = "payroll"``.
