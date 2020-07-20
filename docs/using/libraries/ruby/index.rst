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
You can pass an instance of any Ruby class into oso and access its methods and fields from your policy (see :ref:`application-types`).

Ruby instances can be constructed from inside an oso policy using the :ref:`operator-new` operator if the Ruby class has been **registered** using
either the ``#register_class`` method. An example of this can be found :ref:`here <application-types>`).


Numbers and Booleans
^^^^^^^^^^^^^^^^^^^^
Polar supports both integer and floating point numbers, as well as booleans (see :ref:`basic-types`)

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
    Polar does not support methods that mutate strings in place.

Lists
^^^^^
Ruby `Arrays <https://ruby-doc.org/core/Array.html>`_ are mapped to Polar :ref:`Lists <lists>`. Ruby's Array methods may be accessed from policies:

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
			# ...
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

Hashes
^^^^^^
Ruby hashes are mapped to Polar :ref:`dictionaries`:

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
Oso handles Ruby `enumerators <https://ruby-doc.org/core/Enumerator.html>`_ by evaluating the
yielded values one at a time.

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

In the policy above, the body of the `allow` rule will first evaluate ``"HR" = "payroll"`` and then
``"payroll" = "payroll"``. Because the latter evaluation succeeds, the call to ``Oso#allow`` will succeed.
Note that if ``#get_group`` returned an array instead of an enumerator, the rule would fail because it would be comparing an array (``["HR", "payroll"]``) against a string (``"payroll"``).

Summary
^^^^^^^

.. list-table:: Ruby -> Polar Types Summary
   :widths: 500 500
   :header-rows: 1

   * - Ruby type
     - Polar type
   * - Integer
     - Number (Integer)
   * - Float
     - Number (Float)
   * - TrueClass
     - Boolean
   * - FalseClass
     - Boolean
   * - Array
     - List
   * - Hash
     - Dictionary
