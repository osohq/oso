require 'oso'
require 'sequel'
require 'sinatra'

require_relative './models'

DB = Sequel.sqlite

DB.create_table :repositories do
  primary_key :name
  String :name
  TrueClass :is_public
end

repositories = DB[:repositories]

class Repository < Sequel::Model(:repositories)
end

oso = Oso.new
oso.register_class(User)
oso.register_class(
  Repository
