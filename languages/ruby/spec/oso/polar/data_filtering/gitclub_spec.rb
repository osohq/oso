# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'
DB_FILE = 'gitclub_test.db'

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  context 'a github clone' do # rubocop:disable Metrics/BlockLength
    context 'org members' do
      it 'can access the right resources' do
        # steve is a member of osohq
        check_authz steve, 'read', Org, [osohq]
        check_authz steve, 'list_repos', Org, [osohq]
        check_authz steve, 'create_repos', Org, []

        check_authz steve, 'read', Repo, [oso, demo]
        check_authz steve, 'push', Repo, []
        check_authz steve, 'pull', Repo, [oso, demo]
        check_authz steve, 'create_issues', Repo, []
        check_authz steve, 'list_issues', Repo, [oso, demo]

        check_authz steve, 'read', Issue, [bug]
        check_authz steve, 'edit', Issue, []
      end
    end

    context 'org owners' do
      it 'can access the right resources' do
        # leina is an owner of osohq
        check_authz leina, 'read', Org, [osohq]
        check_authz leina, 'list_repos', Org, [osohq]
        check_authz leina, 'create_repos', Org, [osohq]

        check_authz leina, 'read', Repo, [oso, demo]
        check_authz leina, 'push', Repo, [oso, demo]
        check_authz leina, 'pull', Repo, [oso, demo]
        check_authz leina, 'create_issues', Repo, [oso, demo]
        check_authz leina, 'list_issues', Repo, [oso, demo]

        check_authz leina, 'read', Issue, [bug]
        check_authz leina, 'edit', Issue, [bug]
      end
    end

    context 'repo readers' do
      it 'can access the right resources' do
        # graham owns apple and has read access to demo
        check_authz graham, 'read', Org, [apple]
        check_authz graham, 'list_repos', Org, [apple]
        check_authz graham, 'create_repos', Org, [apple]

        check_authz graham, 'read', Repo, [ios, demo]
        check_authz graham, 'push', Repo, [ios]
        check_authz graham, 'pull', Repo, [ios, demo]
        check_authz graham, 'create_issues', Repo, [ios]
        check_authz graham, 'list_issues', Repo, [ios, demo]

        check_authz graham, 'read', Issue, [laggy]
        check_authz graham, 'edit', Issue, [laggy]
      end
    end

    context 'repo writers' do
      it 'can access the right resources' do
        # gabe has write access to oso
        check_authz gabe, 'read', Org, []
        check_authz gabe, 'list_repos', Org, []
        check_authz gabe, 'create_repos', Org, []

        check_authz gabe, 'read', Repo, [oso]
        check_authz gabe, 'push', Repo, [oso]
        check_authz gabe, 'pull', Repo, [oso]
        check_authz gabe, 'create_issues', Repo, [oso]
        check_authz gabe, 'list_issues', Repo, [oso]

        check_authz gabe, 'read', Issue, [bug]
        check_authz gabe, 'edit', Issue, [bug]
      end
    end
    context 'issue reviewer' do
      it 'can access the right resources' do
        check_authz sam, 'edit', Issue, [laggy]
      end
    end
  end

  let(:policy_file) { File.join(__dir__, 'gitclub.polar') }
  let(:apple) { Org.find 'apple' }
  let(:osohq) { Org.find 'osohq' }

  let(:oso) { Repo.find 'oso' }
  let(:demo) { Repo.find 'demo' }
  let(:ios) { Repo.find 'ios' }

  let(:steve) { User.find 'steve' }
  let(:leina) { User.find 'leina' }
  let(:sam) { User.find 'sam' }
  let(:gabe) { User.find 'gabe' }
  let(:graham) { User.find 'graham' }

  let(:bug) { Issue.find 'bug' }
  let(:laggy) { Issue.find 'laggy' }

  before do # rubocop:disable Metrics/BlockLength
    File.delete DB_FILE if File.exist? DB_FILE
    SQLite3::Database.new(DB_FILE) do |db| # rubocop:disable Metrics/BlockLength
      db.execute <<-SQL
        create table orgs (
          name varchar(16) not null primary key
        );
      SQL

      db.execute <<-SQL
        create table users (
          name varchar(16) not null primary key,
          org_name varchar(16) not null
        );
      SQL

      db.execute <<-SQL
        create table repos (
          name varchar(16) not null primary key,
          org_name varchar(16) not null
        );
      SQL

      db.execute <<-SQL
        create table issues (
          name varchar(16) not null primary key,
          repo_name varchar(16) not null,
          reviewer_name varchar(16)
        );
      SQL

      db.execute <<-SQL
        create table repo_roles (
          id integer not null primary key autoincrement,
          name varchar(16) not null,
          repo_name varchar(16) not null,
          user_name varchar(16) not null
        );
      SQL

      db.execute <<-SQL
        create table org_roles (
          id integer not null primary key autoincrement,
          name varchar(16) not null,
          org_name varchar(16) not null,
          user_name varchar(16) not null
        );
      SQL
    end

    ActiveRecord::Base.establish_connection(
      adapter: 'sqlite3',
      database: DB_FILE
    )

    # fixtures
    apple = Org.create name: 'apple'
    osohq = Org.create name: 'osohq'

    oso = Repo.create name: 'oso', org: osohq
    demo = Repo.create name: 'demo', org: osohq
    ios = Repo.create name: 'ios', org: apple

    steve = User.create name: 'steve', org: osohq
    sam = User.create name: 'sam', org: osohq
    leina = User.create name: 'leina', org: osohq
    gabe = User.create name: 'gabe', org: osohq
    graham = User.create name: 'graham', org: apple

    OrgRole.create name: 'owner', user: leina, org: osohq
    OrgRole.create name: 'member', user: steve, org: osohq
    OrgRole.create name: 'owner', user: graham, org: apple

    RepoRole.create name: 'writer', user: gabe, repo: oso
    RepoRole.create name: 'reader', user: graham, repo: demo

    Issue.create name: 'bug', repo: oso
    Issue.create name: 'laggy', repo: ios, reviewer: sam

    subject.register_class(
      User,
      fields: {
        name: String,
        org_name: String,
        org: Relation.new(
          kind: 'one',
          other_type: 'Org',
          my_field: 'org_name',
          other_field: 'name'
        )
      }
    )

    subject.register_class(
      Org,
      fields: {
        name: String,
        users: Relation.new(
          kind: 'many',
          other_type: 'User',
          my_field: 'name',
          other_field: 'org_name'
        ),
        repos: Relation.new(
          kind: 'many',
          other_type: 'Repo',
          my_field: 'name',
          other_field: 'org_name'
        )
      }
    )

    subject.register_class(
      Repo,
      fields: {
        name: String,
        org_name: String,
        org: Relation.new(
          kind: 'one',
          other_type: 'Org',
          my_field: 'org_name',
          other_field: 'name'
        ),
        roles: Relation.new(
          kind: 'many',
          other_type: 'Role',
          my_field: 'name',
          other_field: 'user_name'
        )
      }
    )

    subject.register_class(
      Issue,
      fields: {
        name: String,
        repo_name: String,
        repo: Relation.new(
          kind: 'one',
          other_type: 'Repo',
          my_field: 'repo_name',
          other_field: 'name'
        ),
        reviewer: Relation.new(
          kind: 'one',
          other_type: 'User',
          my_field: 'reviewer_name',
          other_field: 'name'
        )
      }
    )

    subject.data_filtering_adapter =
      ::Oso::Polar::Data::Adapter::ActiveRecordAdapter.new

    subject.load_files [policy_file]
  end
end

class User < ActiveRecord::Base
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :org_roles, foreign_key: :user_name
  has_many :repo_roles, foreign_key: :user_name
end

class Repo < ActiveRecord::Base
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :issues, foreign_key: :repo_name
  has_many :repo_roles, foreign_key: :repo_name
end

class Org < ActiveRecord::Base
  self.primary_key = :name
  has_many :users, foreign_key: :org_name
  has_many :repos, foreign_key: :org_name
  has_many :org_roles, foreign_key: :org_name
end

class Issue < ActiveRecord::Base
  self.primary_key = :name
  belongs_to :repo, foreign_key: :repo_name
  belongs_to :reviewer, class_name: 'User', foreign_key: 'reviewer_name', optional: true
end

class RepoRole < ActiveRecord::Base
  belongs_to :user, foreign_key: :user_name
  belongs_to :repo, foreign_key: :repo_name
end

class OrgRole < ActiveRecord::Base
  belongs_to :user, foreign_key: :user_name
  belongs_to :org, foreign_key: :org_name
end
