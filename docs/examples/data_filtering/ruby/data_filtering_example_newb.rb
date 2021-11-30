# docs: begin-b1
require 'active_record'
require 'sqlite3'
require 'oso'
require 'oso/polar/data/adapter/active_record_adapter'

DB_FILE = '/tmp/test.db'
Relation = Oso::Relation

class Organization < ActiveRecord::Base
end

class Repository < ActiveRecord::Base
  belongs_to :organization, foreign_key: :org_id
end

class User < ActiveRecord::Base
  has_many :repo_roles
  has_many :org_roles
end

class OrgRole < ActiveRecord::Base
  belongs_to :user
  belongs_to :organization, foreign_key: :org_id
end

class RepoRole < ActiveRecord::Base
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
  oso.data_filtering_adapter = ::Oso::Polar::Data::Adapter::ActiveRecordAdapter.new

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
  raise unless results == [demo_repo, oso_repo]
end

example
# docs: end-b3
