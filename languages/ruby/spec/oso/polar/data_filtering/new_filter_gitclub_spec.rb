# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength

  context 'new filters' do

    context 'gitclub' do
      it 'pls' do
        filter = subject.authzd_query gabe, 'read', Repo
        expect(filter.to_a).to eq [oso]
      end
      it 'and also' do
        filter = subject.authzd_query gabe, 'read', Issue
        expect(filter.to_a).to eq [bug]
      end

      DB_FILE = 'gitclub_test.db'
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
              repo_name varchar(16) not null
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
        osohq = Org.create name: 'oso'

        oso = Repo.create name: 'oso', org: osohq
        demo = Repo.create name: 'demo', org: osohq
        ios = Repo.create name: 'ios', org: apple

        steve = User.create name: 'steve', org: osohq
        leina = User.create name: 'leina', org: osohq
        gabe = User.create name: 'gabe', org: osohq
        graham = User.create name: 'graham', org: apple

        OrgRole.create name: 'owner', user: leina, org: osohq
        OrgRole.create name: 'member', user: steve, org: osohq
        OrgRole.create name: 'owner', user: graham, org: apple

        RepoRole.create name: 'writer', user: gabe, repo: oso
        RepoRole.create name: 'reader', user: graham, repo: demo

        Issue.create name: 'bug', repo: oso
        Issue.create name: 'laggy', repo: ios
        Issue.create name: 'more polar adventure endings', repo: demo

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
            )
          }
        )

        policy_file = File.join(__dir__, 'gitclub.polar')
        subject.load_files [policy_file]
      end
      let(:apple) { Org.find 'apple' }
      let(:osohq) { Org.find 'oso' }
      let(:oso) { Repo.find 'oso' }
      let(:demo) { Repo.find 'demo' }
      let(:ios) { Repo.find 'ios' }
      let(:steve) { User.find 'steve' }
      let(:leina) { User.find 'leina' }
      let(:gabe) { User.find 'gabe' }
      let(:graham) { User.find 'graham' }
      let(:bug) { Issue.find 'bug' }
      let(:laggy) { Issue.find 'laggy' }
      let(:endings) { Issue.find 'more polar adventure endings' }
    end
  end
end

class User < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :org_roles, foreign_key: :user_name
  has_many :repo_roles, foreign_key: :user_name
end

class Repo < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :issues, foreign_key: :repo_name
  has_many :repo_roles, foreign_key: :repo_name
end

class Org < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  has_many :users, foreign_key: :org_name
  has_many :repos, foreign_key: :org_name
  has_many :org_roles, foreign_key: :org_name
end

class Issue < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :repo, foreign_key: :repo_name
end

class RepoRole < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  belongs_to :user, foreign_key: :user_name
  belongs_to :repo, foreign_key: :repo_name
end

class OrgRole < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  belongs_to :user, foreign_key: :user_name
  belongs_to :org, foreign_key: :org_name
end
