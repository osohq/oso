---
instance: instance
isAdmin: is_admin
isAdminOf: admin_of?
isAllowed: allowed?
langName: Ruby
startswith: start_with?
userParams: "\"alice\", is_admin: true"

classMethodExample: |
  ```ruby
  class User
    # ...
    def self.superusers
      ["alice", "bhavik", "clarice"]
    end
  end

  OSO.register_class(User)

  user = User.new("alice", is_admin: true)
  raise "should be allowed" unless OSO.allowed?(actor: user, action: "foo", resource: "bar")
  ```

registerClass: |
  <!-- TODO(gj): link to API docs when available. -->
  Ruby classes are registered using `register_class()`:

  ```ruby
  OSO.register_class(User)
  ```

specializedExample: |
  ```ruby
  OSO.register_class(User)
  user = User.new("alice", is_admin: true)
  raise "should be allowed" unless OSO.allowed?(actor: user, action: "foo", resource: "bar")
  raise "should not be allowed" unless not OSO.allowed?(actor: user, action: "foo", resource: "bar")
  ```

testQueries: |
  ```polar
  ?= allow(new User("alice", is_admin: true), "foo", "bar");
  ```

userClass: |
  ```ruby
  class User
    attr_reader :name
    attr_reader :is_admin

    def initialize(name, is_admin:)
      @name = name
      @is_admin = is_admin
    end
  end

  user = User.new("alice", is_admin: true)
  raise "should be allowed" unless OSO.allowed?(actor: user, action: "foo", resource: "bar")
  ```
---
