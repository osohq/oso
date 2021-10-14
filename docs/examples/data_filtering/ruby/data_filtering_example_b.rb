# docs: begin-b1
require 'active_record'
require 'sqlite3'
require 'oso'

DB_FILE = '/tmp/test.db'
Relation = Oso::Relation

class Organization < ActiveRecord::Base
  include QueryConfig
end

class Repository < ActiveRecord::Base
  include QueryConfig
  belongs_to :organization, foreign_key: :org_id
end

class User < ActiveRecord::Base
  include QueryConfig
  has_many :repo_roles
  has_many :org_roles
end

class OrgRole < ActiveRecord::Base
  include QueryConfig
  belongs_to :user
  belongs_to :organization, foreign_key: :org_id
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
    create table organizations (
      id varchar(16) not null primary key
    );
  SQL

  db.execute <<-SQL
    create table users (
      id varchar(16) not null primary key
    );
  SQL

  db.execute <<-SQL
    create table repositories (
      id varchar(16) not null primary key,
      org_id varchar(16) not null
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

  db.execute <<-SQL
    create table org_roles (
      id integer not null primary key autoincrement,
      name varchar(16) not null,
      org_id varchar(16) not null,
      user_id varchar(16) not null
    );
  SQL

  ActiveRecord::Base.establish_connection(
    adapter: 'sqlite3',
    database: DB_FILE
  )
end
# docs: end-b1

# docs: begin-b2
def init_oso
  oso = Oso.new

  oso.register_class(
    Organization,
    fields: { id: String }
  )

  oso.register_class(
    Repository,
    fields: {
      id: String,
      organization: Relation.new(
        kind: 'one',
        other_type: Organization,
        my_field: 'org_id',
        other_field: 'id'
      )
    }
  )

  oso.register_class(
    User,
    fields: { id: String, }
  )

  oso.register_class(
    RepoRole,
    fields: { name: String, }
  )

  oso.register_class(
    OrgRole,
    fields: { name: String, }
  )

  oso
end

module QueryConfig
  def self.included(base)
    base.instance_eval do

      # Turn a constraint into a param hash for #where
      query_clause = lambda do |f|
        if f.field.nil?
          { primary_key => f.value.send(primary_key) }
        else
          { f.field => f.value }
        end
      end

      # ActiveRecord automatically turns array values in where clauses into
      # IN conditions, so Eq and In can share the same code.
      @filter_handlers = {
        'Eq'  => ->(query, filter) { query.where     query_clause[filter] },
        'In'  => ->(query, filter) { query.where     query_clause[filter] },
        'Neq' => ->(query, filter) { query.where.not query_clause[filter] }
      }

      @filter_handlers.default_proc = proc do |k|
        raise "Unsupported filter kind: #{k}"
      end

      @filter_handlers.freeze

      # Create a query from an array of filters
      def self.build_query(filters)
        filters.reduce(all) do |query, filter|
          @filter_handlers[filter.kind][query, filter]
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
# docs: end-b2

# docs: begin-b3
def example
  init_db
  oso = init_oso

  osohq = Organization.create id: 'osohq'
  apple = Organization.create id: 'apple'

  ios = Repository.create id: 'ios', organization: apple
  oso_repo = Repository.create id: 'oso', organization: osohq
  demo_repo = Repository.create id: 'demo', organization: osohq

  leina = User.create id: 'leina'
  steve = User.create id: 'steve'

  OrgRole.create user: leina, organization: osohq, name: 'owner'

  oso.load_files(['policy_b.polar'])

  results = oso.authorized_resources(leina, 'read', Repository)
  raise unless results == [oso_repo, demo_repo]
end

example
# docs: end-b3
