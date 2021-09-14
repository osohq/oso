# frozen_string_literal: true

require 'set'

require 'oso'

# docs: begin-types
class Organization
  attr_reader :name

  def initialize(name)
    @name = name
  end
end

class Repository
  attr_reader :name, :organization

  def initialize(name, organization)
    @name = name
    @organization = organization
  end
end

class Role
  attr_reader :name, :resource

  def initialize(name, resource)
    @name = name
    @resource = resource
  end
end

class User
  attr_reader :name, :roles

  def initialize(name)
    @name = name
    @roles = Set.new
  end

  def assign_role_for_resource(name, resource)
    self.roles.add Role.new(name, resource)
  end
end
# docs: end-types

# docs: begin-setup
oso = Oso.new

# docs: begin-register
oso.register_class Organization
oso.register_class Repository
oso.register_class User
# docs: end-register

oso.load_files ['main.polar']
# docs: end-setup
