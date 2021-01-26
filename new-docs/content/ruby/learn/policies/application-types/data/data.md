---
instance: instance
isAdmin: is_admin
isAdminOf: admin_of?
isAllowed: allowed?
langName: Ruby
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
