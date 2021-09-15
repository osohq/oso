require 'oso'
require 'sequel'
require 'sinatra'

require_relative './models'

DB = Sequel.sqlite

DB.create_table :repositories do
  String :name
  TrueClass :is_public
end

repositories = DB[:repositories]

module DF
  class Repository < Sequel::Model(:repositories)
  end

  r = Repository.new(name: "gmail", is_public: false)
  r.save

  OSO = Oso.new

  # docs: begin-data-filtering
  # This is an example implementation for the Sequel ORM, but you can
  # use any ORM with this API.
  def self.get_repositories(filters)
    query = Repository
    filters.each do |filter|
      value = filter.value

      if filter.field.nil?
        value = value.name
        field = :name
      else
        field = filter.field.to_sym
      end

      if filter.kind == "Eq"
        query = query.where(field => value)
      else
        raise "unimplemented constraint kind #{filter.kind}"
      end
    end

    query
  end

  def self.combine_query(q1, q2)
    q1.union(q2)
  end

  def self.exec_query(q)
    q.all
  end

  OSO.register_class(User)
  OSO.register_class(
    Repository,
    name: "Repository",
    fields: {
      is_public: PolarBoolean
    },
    build_query: method(:get_repositories),
    combine_query: method(:combine_query),
    exec_query: method(:exec_query)
  )

  OSO.load_files(["main.polar"])
  # docs: end-data-filtering


end

def get_current_user
  User.new([{name: "admin", repository: DF::Repository.new(name: "gmail")}])
end

def serialize(repositories)
  repositories.to_s
end

oso = DF::OSO

# docs: begin-list-route
get "/repos" do
  repositories = oso.authorized_resources(
    get_current_user(),
    "read",
    Repository)

  serialize(repositories)
end
# docs: end-list-route
