# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'

D = Oso::Polar::Data
RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength

  context 'new filters' do
    # These names are to make it easy to write new filters for tests.
    # But in actual use we'll never need to manually construct these,
    # they'll be deserialized from the output of `build_filter_plan`
    Join = D::ArelJoin
    Src = D::ArelSource
    Select = D::ArelSelect
    Field = D::Proj
    Value = D::Value

    repos = Src[Repo]
    users = Src[User]
    issues = Src[Issue]
    orgs = Src[Org]
    user_name = Field[users, :name]
    org_name = Field[orgs, :name]
    repo_name = Field[repos, :name]
    repo_org_name = Field[repos, :org_name]
    issue_repo_name = Field[issues, :repo_name]
    repos_orgs = Join[repos, repo_org_name, org_name, orgs]
    issues_repos = Join[issues, issue_repo_name, repo_name, repos]
    issues_repos_orgs = Join[issues_repos, repo_org_name, org_name, orgs]

    context 'gitclub' do
      it 'field value no join' do
        # user.name = 'steve'
        result = Select[users, user_name, Value['steve']].to_a
        expect(result).to eq [steve]

        # user.name != 'steve'
        result = Select[users, user_name, Value['steve'], kind: :neq].to_a
        expect(result).to contain_exactly(*[leina, gabe, graham])
      end

      it 'field field no join' do
        # repo.name = repo.org_name
        result = Select[repos, repo_name, repo_org_name].to_a
        expect(result).to eq [oso]

        # repo.name != repo.org_name
        result = Select[repos, repo_name, repo_org_name, kind: :neq].to_a
        expect(result).to contain_exactly(*[demo, ios])
      end

      it 'field value one join' do
        # repo.org.name = 'oso'
        result = Select[repos_orgs, org_name, Value['oso']].to_a
        expect(result).to contain_exactly(*[oso, demo])

        # repo.org.name != 'oso'
        result = Select[repos_orgs, org_name, Value['oso'], kind: :neq].to_a
        expect(result).to contain_exactly(*[ios])
      end

      it 'field field one join' do
        # repo.name = repo.org.name
        result = Select[repos_orgs, repo_name, org_name].to_a
        expect(result).to contain_exactly(*[oso]) # osoroboroso

        # repo.name != repo.org.name
        result = Select[repos_orgs, repo_name, org_name, kind: :neq].to_a
        expect(result).to contain_exactly(*[demo, ios]) # aneponymous
      end

      it 'field value two joins' do
        # issue.repo.org.name = 'apple'
        result = Select[issues_repos_orgs, org_name, Value['apple']].to_a
        expect(result).to eq [laggy]

        # issue.repo.org.name != 'apple'
        result = Select[issues_repos_orgs, org_name, Value['apple'], kind: :neq].to_a
        expect(result).to contain_exactly(*[bug, endings])
      end

      it 'field field two joins' do
        # issue.repo.name = issue.repo.org.name
        result = Select[issues_repos_orgs, repo_name, org_name].to_a
        expect(result).to eq [bug]

        # issue.repo.name != issue.repo.org.name
        result = Select[issues_repos_orgs, repo_name, org_name, kind: :neq].to_a
        expect(result).to contain_exactly(*[laggy, endings])
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
