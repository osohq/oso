# frozen_string_literal: true

module RolesHelpers
  Org = Struct.new :name
  Repo = Struct.new :name, :org_name
  Issue = Struct.new :name, :repo_name
  User = Struct.new :name 
  Role = Struct.new :user_name, :resource_name, :role
end
