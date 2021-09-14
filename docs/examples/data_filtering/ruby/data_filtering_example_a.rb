# docs: begin-a1
# We'll use ActiveRecord in this example, but data filtering can be used with any ORM
require 'active_record'
require 'sqlite3'
require 'oso'

DB_FILE = '/tmp/test.db'
Relation = Oso::Polar::DataFiltering::Relation

class Repository < ActiveRecord::Base
  include QueryConfig # This module adds build/exec/combine query functions for the class
end

class User < ActiveRecord::Base
  include QueryConfig
  has_many :repo_roles
end
class RepoRole < ActiveRecord::Base
  include QueryConfig
  belongs_to :user
  belongs_to :repository, foreign_key: :repo_id
end

def init_db
  File.delete DB_FILE if File.exist? DB_FILE

  db = SQLite3::Database.new(DB_FILE)
  db.execute <<-SQL
    create table users (
      id varchar(16) not null primary key
    );
  SQL

  db.execute <<-SQL
    create table repositories (
      id varchar(16) not null primary key
    );
  SQL

  db.execute <<-SQL
    create table repo_roles (
      id integer not null primary key autoincrement,
      name varchar(16) not null,
      repo_id varchar(16) not null,
      user_id varchar(16) not null
    );
  SQL

  ActiveRecord::Base.establish_connection(
    adapter: 'sqlite3',
    database: DB_FILE
  )
end
# docs: end-a1

# docs: begin-a2
def init_oso
  oso = Oso.new

  oso.register_class(
    Repository,
    fields: { id: String, }
  )

  oso.register_class(
    User,
    fields: { id: String, }
  )

  oso.register_class(
    RepoRole,
    fields: { name: String, }
  )

  oso
end

# We'll use this mixin to automatically supply query functions
# for register_class.
module QueryConfig
  def self.included(base)
    base.instance_eval do

      # Turn a constraint into a param hash for #where
      query_clause = lambda do |c|
        if c.field.nil?
          { primary_key => c.value.send(primary_key) }
        else
          { c.field => c.value }
        end
      end

      # ActiveRecord automatically turns array values in where clauses into
      # IN conditions, so Eq and In can share the same code.
      @constraint_handlers = {
        'Eq'  => ->(query, constraint) { query.where     query_clause[constraint] },
        'In'  => ->(query, constraint) { query.where     query_clause[constraint] },
        'Neq' => ->(query, constraint) { query.where.not query_clause[constraint] }
      }

      @constraint_handlers.default_proc = proc do |k|
        raise "Unsupported constraint kind: #{k}"
      end

      @constraint_handlers.freeze

      # Create a query from an array of constraints
      def self.build_query(constraints)
        constraints.reduce(all) do |query, constraint|
          @constraint_handlers[constraint.kind][query, constraint]
        end
      end

      # Produce an array of values from a query
      def self.exec_query(query)
        query.distinct.to_a
      end

      # Merge two queries into a new query with the results from both
      def self.combine_query(one, two)
        one.or(two)
      end
    end
  end
end

# docs: end-a2

# docs: begin-a3
def example
  init_db
  oso = init_oso

  ios = Repository.create id: 'ios'
  oso_repo = Repository.create id: 'oso'
  demo_repo = Repository.create id: 'demo'

  leina = User.create id: 'leina'
  steve = User.create id: 'steve'

  RepoRole.create user: leina, repository: oso_repo, name: 'contributor'
  RepoRole.create user: leina, repository: demo_repo, name: 'maintainer'

  oso.load_files(['policy_a.polar'])

  results = oso.authorized_resources(leina, 'read', Repository)
  raise unless results == [demo_repo, oso_repo]
end

example
# docs: end-a3
