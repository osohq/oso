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
  class SequelAdapter
    def build_query(filter)
      types = filter.types
      query = filter.relations.reduce(filter.model) do |q, rel|
        rec = types[rel.left].fields[rel.name]
        q.join( rel.right.table_name,
          "#{rel.left.table_name}.#{rec.my_field}" =>
        "#{rel.right.table_name}.#{rec.other_field}"
        )
      end

      args = []
      sql = filter.conditions.map do |conjs|
        conjs.reduce('true') do |sql, conj|
          "(#{sql} AND #{sqlize(conj, args)})"
        end
      end.reduce('false') do |sql, clause|
        "(#{sql} OR #{clause})"
      end 

      query.where(Sequel.lit(sql, *args)).distinct
    end

    def execute_query(query)
      query.to_a
    end

    OPS = {
      'Eq' => '=', 'In' => 'IN', 'Nin' => 'NOT IN', 'Neq' => '!=',
      'Lt' => '<', 'Gt' => '>', 'Leq' => '<=', 'Geq' => '>='
    }.freeze

    private

    def sqlize(cond, args)
      lhs = add_side cond.left, args
      rhs = add_side cond.right, args
      "#{lhs} #{OPS[cond.cmp]} #{rhs}"
    end

    def add_side(side, args)
      if side.is_a? ::Oso::Polar::Data::Filter::Projection
        "#{side.source.table_name}.#{side.field || :name}"
      elsif side.is_a? DF::Repository
        args.push side.name
        '?'
      else
        args.push side
        '?'
      end
    end
  end

  OSO.register_class(User)
  OSO.register_class(
    Repository,
    name: "Repository",
    fields: {
      is_public: PolarBoolean
    },
  )

  OSO.data_filtering_adapter = SequelAdapter.new

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
    DF::Repository)

  serialize(repositories)
end
# docs: end-list-route
