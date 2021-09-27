# frozen_string_literal: true

require 'tempfile'

require_relative './helpers'
require_relative './data_filtering_helpers'

RSpec.configure do |c|
  c.include DataFilteringHelpers
end

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  DFH = DataFilteringHelpers
  Relation = ::Oso::Relation
  Bar = DFH.record(:id, :is_cool, :is_still_cool) do
    def foos
      Foo.all.select { |foo| id == foo.bar_id }
    end
  end

  Foo = DFH.record(:id, :bar_id, :is_fooey, :numbers) do
    def bar
      Bar.all.find { |bar| bar.id == bar_id }
    end
  end

  Log = DFH.record(:id, :foo_id, :data) do
    def foo
      Foo.all.find { |foo| foo.id == foo_id }
    end
  end

  Foo.new('something', 'hello', false, [])
  Foo.new('another', 'hello', true, [1])
  Foo.new('third', 'hello', true, [2])
  Foo.new('fourth', 'goodbye', true, [2, 1, 0])

  Bar.new('hello', true, true)
  Bar.new('goodbye', false, true)
  Bar.new('hershey', false, false)

  Log.new('a', 'fourth', 'hello')
  Log.new('b', 'third', 'world')
  Log.new('c', 'another', 'steve')

  Widget = DFH.record :id
  0.upto(9).each { |id| Widget.new id }

  context '#authorized_resources' do # rubocop:disable Metrics/BlockLength
    it 'handles classes with explicit names' do
      subject.register_class(
        Widget,
        name: 'Doohickey',
        fields: { id: Integer }
      )

      subject.load_str 'allow("gwen", "get", it: Doohickey) if it.id = 8;'
      check_authz 'gwen', 'get', Widget, [Widget.all[8]]
    end

    it 'handles queries that return known results' do
      subject.register_class(Widget, fields: { id: Integer })
      subject.register_constant(Widget.all.first, name: 'Prototype')
      subject.load_str <<~POL
        allow("gwen", "tag", w: Widget) if w = Prototype;
        allow("gwen", "tag", w: Widget) if w.id in [1,2,3];
      POL
      results = subject.authorized_resources 'gwen', 'tag', Widget

      expect(results).to contain_exactly(*Widget.all[0..3])
    end

    context 'when filtering data' do # rubocop:disable Metrics/BlockLength
      before do # rubocop:disable Metrics/BlockLength
        subject.register_class(
          Bar,
          fields: {
            id: String,
            is_cool: PolarBoolean,
            is_still_cool: PolarBoolean,
            foos: Relation.new(
              kind: 'many',
              other_type: 'Foo',
              my_field: 'id',
              other_field: 'bar_id'
            )
          }
        )

        subject.register_class(
          Log,
          fields: {
            id: String,
            foo_id: String,
            data: String,
            foo: Relation.new(
              kind: 'one',
              other_type: 'Foo',
              my_field: 'foo_id',
              other_field: 'id'
            )
          }
        )

        subject.register_class(
          Foo,
          fields: {
            id: String,
            bar_id: String,
            is_fooey: PolarBoolean,
            numbers: Array,
            bar: Relation.new(
              kind: 'one',
              other_type: 'Bar',
              my_field: 'bar_id',
              other_field: 'id'
            ),
            logs: Relation.new(
              kind: 'many',
              other_type: 'Log',
              my_field: 'id',
              other_field: 'foo_id'
            )
          }
        )
      end

      context 'with specializers in the rule head' do # rubocop:disable Metrics/BlockLength
        it 'works' do # rubocop:disable Metrics/BlockLength
          subject.load_str <<~POL
            allow(foo: Foo,             "NoneNone", log) if foo = log.foo;
            allow(foo,                  "NoneCls",  log: Log) if foo = log.foo;
            allow(foo,                  "NoneDict", _: {foo:foo});
            allow(foo,                  "NonePtn",  _: Log{foo: foo});
            allow(foo: Foo,             "ClsNone",  log) if log in foo.logs;
            allow(foo: Foo,             "ClsCls",   log: Log) if foo = log.foo;
            allow(foo: Foo,             "ClsDict",  _: {foo: foo});
            allow(foo: Foo,             "ClsPtn",   _: Log{foo: foo});
            allow(_: {logs: logs},      "DictNone", log) if log in logs;
            allow(_: {logs: logs},      "DictCls",  log: Log) if log in logs;
            allow(foo: {logs: logs},    "DictDict", log: {foo: foo}) if log in logs;
            allow(foo: {logs: logs},    "DictPtn",  log: Log{foo: foo}) if log in logs;
            allow(_: Foo{logs: logs},   "PtnNone",  log) if log in logs;
            allow(_: Foo{logs: logs},   "PtnCls",   log: Log) if log in logs;
            allow(foo: Foo{logs: logs}, "PtnDict",  log: {foo: foo}) if log in logs;
            allow(foo: Foo{logs: logs}, "PtnPtn",   log: Log{foo: foo}) if log in logs;
          POL
          parts = %w[None Cls Dict Ptn]
          parts.each do |a|
            parts.each do |b|
              Log.all.each do |log|
                results = subject.authorized_resources log.foo, a + b, Log
                expect(results).to contain_exactly(log)
              end
            end
          end
        end
      end

      context 'for collection membership' do # rubocop:disable Metrics/BlockLength
        it 'can check if a value is in a field' do
          policy = 'allow("gwen", "get", foo: Foo) if 1 in foo.numbers and 2 in foo.numbers;'
          subject.load_str(policy)
          results = subject.authorized_resources('gwen', 'get', Foo)
          expected = Foo.all.select { |f| f.numbers.include?(1) and f.numbers.include?(2) }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can check if a field is in a value' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.numbers in [[1]];'
          subject.load_str(policy)
          results = subject.authorized_resources('gwen', 'get', Foo)
          expected = Foo.all.select { |f| f.numbers == [1] }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can check if a value is in a field on a direct relation' do
          policy = 'allow("gwen", "get", log: Log) if 1 in log.foo.numbers;'
          subject.load_str policy
          results = subject.authorized_resources('gwen', 'get', Log)
          expected = Log.all.select { |l| l.foo.numbers.include? 1 }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can check if a value is in a field on an indirect relation' do
          subject.load_str <<~POL
            allow("gwen", "get", log: Log) if
              foo in log.foo.bar.foos and
              0 in foo.numbers;
          POL
          results = subject.authorized_resources('gwen', 'get', Log)
          expected = Log.all.select { |l| l.foo.bar.foos.any? { |f| f.numbers.include? 0 } }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end
      end

      context 'for equality' do # rubocop:disable Metrics/BlockLength
        it 'can compare a field with a known value' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.is_fooey = true;'
          subject.load_str(policy)
          results = subject.authorized_resources('gwen', 'get', Foo)
          expected = Foo.all.select(&:is_fooey)
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on the same object' do
          subject.load_str <<~POL
            allow("gwen", "put", bar: Bar) if
              bar.is_cool = bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Bar)
          expected = Bar.all.select { |b| b.is_cool == b.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on a related object' do
          subject.load_str <<~POL
            allow("gwen", "put", foo: Foo) if
              foo.bar.is_cool = foo.bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Foo)
          expected = Foo.all.select { |foo| foo.bar.is_cool == foo.bar.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on an indirectly related object' do
          subject.load_str <<~POL
            allow("gwen", "put", log: Log) if
              log.data = "world" and
              log.foo.bar.is_cool = log.foo.bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Log)
          expected = Log.all.select do |log|
            log.data == 'world' and log.foo.bar.is_still_cool == log.foo.bar.is_cool
          end
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'returns empty results for an impossible query' do
          subject.load_str <<~POL
            allow("gwen", "gwt", foo: Foo) if
              foo.is_fooey = true and
              foo.is_fooey = false;
          POL

          results = subject.authorized_resources('gwen', 'get', Foo)
          expect(results).to be_empty
        end

        it 'correctly applies constraints from other rules' do
          subject.load_str <<~POL
            f(bar: Bar) if bar.is_cool = true;
            g(bar: Bar) if bar.is_still_cool = true;
            h(bar: Bar) if foo in bar.foos and log in foo.logs and i(log);
            i(log: Log) if log.data = "world";
            allow("gwen", "get", bar: Bar) if
              f(bar) and g(bar) and h(bar);
          POL

          results = subject.authorized_resources('gwen', 'get', Bar)
          expected = Bar.all.find { |bar| bar.id == 'hello' }
          expect(results).to contain_exactly(expected)
        end
      end

      context 'for inequality' do # rubocop:disable Metrics/BlockLength
        it 'can compare two fields on the same object' do
          subject.load_str <<~POL
            allow("gwen", "get", bar: Bar) if
              bar.is_cool != bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'get', Bar)
          expected = Bar.all.reject { |b| b.is_cool == b.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on a related object' do
          subject.load_str <<~POL
            allow("gwen", "put", foo: Foo) if foo.bar.is_cool != foo.bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Foo)
          expected = Foo.all.reject { |foo| foo.bar.is_cool == foo.bar.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on an indirectly related object' do
          policy = <<~POL
            allow("gwen", "put", log: Log) if
              log.data = "hello" and
              log.foo.bar.is_cool != log.foo.bar.is_still_cool;
          POL
          subject.load_str(policy)

          results = subject.authorized_resources('gwen', 'put', Log)
          expected = Log.all.select do |log|
            log.data == 'hello' and log.foo.bar.is_still_cool != log.foo.bar.is_cool
          end
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end
      end

      it 'handles one-to-one relationships' do
        policy = <<~POL
          allow("gwen", "get", foo: Foo) if
            foo.is_fooey = true and
            foo.bar.is_cool = true;
        POL
        subject.load_str(policy)

        results = subject.authorized_resources('gwen', 'get', Foo)
        expected = Foo.all.select { |foo| foo.bar.is_cool and foo.is_fooey }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'handles one-to-many relationships' do
        policy = 'allow("gwen", "get", foo: Foo) if log in foo.logs and log.data = "hello";'
        subject.load_str policy
        expected = Foo.all.select { |foo| foo.id == 'fourth' }
        check_authz 'gwen', 'get', Foo, expected
      end

      it 'handles nested one-to-one relationships' do
        policy = <<~POL
          allow("gwen", "put", log: Log) if
            log.data = "hello" and
            log.foo.is_fooey = true and
            log.foo.bar.is_cool != true;
        POL
        subject.load_str(policy)

        results = subject.authorized_resources('gwen', 'put', Log)
        expected = Log.all.select { |log| log.data == 'hello' and log.foo.is_fooey and !log.foo.bar.is_cool }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'handles all the relationships at once' do
        policy = <<~POL
          allow(log: Log, "a", foo: Foo) if log in foo.logs;
          allow(log: Log, "b", foo: Foo) if foo = log.foo;
          allow(log: Log, "c", foo: Foo) if log.foo = foo and log in foo.logs;
          allow(log: Log, "d", foo: Foo) if log in foo.logs and log.foo = foo;
        POL
        subject.load_str policy
        log = Log.all.find { |l| l.foo_id == 'fourth' }
        foos = Foo.all.select { |foo| foo.id == 'fourth' }
        %w[a b c d].each { |x| check_authz log, x, Foo, foos }
      end
    end
  end

  context 'when meddling with the affairs of wizards' do # rubocop:disable Metrics/BlockLength
    Wizard = DFH.record(:name, :books, :spell_levels) do
      def spells
        Spell.all.select do |spell|
          books.include?(spell.school) and spell_levels.include?(spell.level)
        end
      end
    end

    Familiar = DFH.record :name, :kind, :wizard_name
    Spell = DFH.record :name, :school, :level
    Spell.new('teleport other',    'thaumaturgy', 7)
    Spell.new('wish',              'thaumaturgy', 9)
    Spell.new('cure light wounds', 'necromancy',  1)
    Spell.new('identify',          'divination',  1)
    Spell.new('call familiar',     'summoning',   1)
    Spell.new('call ent',          'summoning',   7)
    Spell.new('magic missile',     'destruction', 1)
    Spell.new('liquify organ',     'destruction', 5)
    Spell.new('call dragon',       'summoning',   9)
    Spell.new('know alignment',    'divination',  6)
    let(:level) { ->(n) { 1.upto(n).to_a } }
    let(:policy_file) { File.join(__dir__, 'magic_policy.polar') }
    let(:gandalf) { Wizard.new('gandalf', %w[divination destruction], level[4]) }
    let(:galadriel) { Wizard.new('galadriel', %w[thaumaturgy divination inscription], level[7]) }
    let(:baba_yaga) { Wizard.new('baba yaga', %w[necromancy summoning destruction], level[8]) }
    let(:shadowfax) { Familiar.new('shadowfax', 'horse', 'gandalf') }
    let(:brown_jenkin) { Familiar.new('brown jenkin', 'rat', 'baba yaga') }
    let(:gimli) { Familiar.new('gimli', 'dwarf', 'galadriel') }
    let(:hedwig) { Familiar.new('hedwig', 'owl', 'galadriel') }

    before do # rubocop:disable Metrics/BlockLength
      subject.register_class(
        Wizard,
        fields: {
          name: String,
          books: Array,
          spell_levels: Array,
          familiars: Relation.new(
            kind: 'many',
            other_type: 'Familiar',
            my_field: 'name',
            other_field: 'wizard_name'
          )
        }
      )

      subject.register_class(
        Spell,
        fields: {
          name: String,
          school: String,
          level: Integer
        }
      )

      subject.register_class(
        Familiar,
        fields: {
          name: String,
          kind: String,
          wizard_name: String,
          wizard: Relation.new(
            kind: 'one',
            other_type: 'Wizard',
            my_field: 'wizard_name',
            other_field: 'name'
          )
        }
      )

      subject.load_files [policy_file]
    end

    context 'wizards' do
      it 'can cast any spell in their spellbook up to their level' do
        Wizard.all.each do |wiz|
          check_authz wiz, 'cast', Spell, wiz.spells
        end
      end

      it 'can ride their horse familiars' do
        check_authz gandalf, 'ride', Familiar, [shadowfax]
        check_authz galadriel, 'ride', Familiar, []
        check_authz baba_yaga, 'ride', Familiar, []
      end

      it 'can groom their familiars' do
        check_authz baba_yaga, 'groom', Familiar, [brown_jenkin]
        check_authz galadriel, 'groom', Familiar, [hedwig, gimli]
        check_authz gandalf, 'groom', Familiar, [shadowfax]
      end

      context 'having mastered inscription' do
        it 'can inscribe any spell they can cast' do
          check_authz galadriel, 'inscribe', Spell, galadriel.spells
          check_authz gandalf, 'inscribe', Spell, []
          check_authz baba_yaga, 'inscribe', Spell, []
        end
      end
    end

    context 'rat familiars' do
      it 'can groom other familiars, except owls (predator)' do
        check_authz brown_jenkin, 'groom', Familiar, [gimli, brown_jenkin, shadowfax]
      end
      it 'can groom their wizard' do
        check_authz brown_jenkin, 'groom', Wizard, [baba_yaga]
      end
    end
  end

  context 'using ActiveRecord' do # rubocop:disable Metrics/BlockLength
    require 'sqlite3'
    require 'active_record'

    DB_FILE = 'active_record_test.db'

    before do
      File.delete DB_FILE if File.exist? DB_FILE
    end

    context 'a github clone' do # rubocop:disable Metrics/BlockLength
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

      before do # rubocop:disable Metrics/BlockLength
        db = SQLite3::Database.new DB_FILE

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

        ActiveRecord::Base.establish_connection(
          adapter: 'sqlite3',
          database: DB_FILE
        )

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

        # fixtures
        apple = Org.create name: 'apple'
        osohq = Org.create name: 'osohq'

        oso_repo = Repo.create name: 'oso', org: osohq
        demo_repo = Repo.create name: 'demo', org: osohq
        ios = Repo.create name: 'ios', org: apple

        steve = User.create name: 'steve', org: osohq
        leina = User.create name: 'leina', org: osohq
        gabe = User.create name: 'gabe', org: osohq
        graham = User.create name: 'graham', org: apple

        OrgRole.create name: 'owner', user: leina, org: osohq
        OrgRole.create name: 'member', user: steve, org: osohq
        OrgRole.create name: 'owner', user: graham, org: apple

        RepoRole.create name: 'writer', user: gabe, repo: oso_repo
        RepoRole.create name: 'reader', user: graham, repo: demo_repo

        Issue.create name: 'bug', repo: oso_repo
        Issue.create name: 'laggy', repo: ios

        subject.load_str <<~POL
          actor User {}

          resource Org {
            roles = ["owner", "member"];
            permissions = ["read", "create_repos", "list_repos"];

            "read" if "member";
            "list_repos" if "member";

            "create_repos" if "owner";

            "member" if "owner";
          }

          resource Repo {
            roles = ["reader", "writer"];
            permissions = ["read", "push", "pull", "create_issues", "list_issues"];
            relations = { parent: Org };

            "read" if "reader";
            "pull" if "reader";
            "list_issues" if "reader";

            "push" if "writer";
            "create_issues" if "writer";

            "reader" if "writer";
            "reader" if "member" on "parent";
            "writer" if "owner" on "parent";
          }

          resource Issue {
            permissions = ["read", "edit"];
            relations = { parent: Repo };
            "read" if "reader" on "parent";
            "edit" if "writer" on "parent";
          }

          has_role(user: User, name: String, org: Org) if
            role in user.org_roles and
            role matches { name: name, org: org };

          has_role(user: User, name: String, repo: Repo) if
            role in user.repo_roles and
            role matches { name: name, repo: repo };

          has_relation(org: Org, "parent", _: Repo{org: org});
          has_relation(repo: Repo, "parent", _: Issue{repo: repo});

          allow(actor, action, resource) if has_permission(actor, action, resource);
        POL
      end

      let(:bug) { Issue.find 'bug' }
      let(:oso) { Repo.find 'oso' }
      let(:demo) { Repo.find 'demo' }
      let(:ios) { Repo.find 'ios' }
      let(:osohq) { Org.find 'osohq' }
      let(:apple) { Org.find 'apple' }
      let(:laggy) { Issue.find 'laggy' }

      let(:steve) { User.find 'steve' }
      let(:leina) { User.find 'leina' }
      let(:gabe) { User.find 'gabe' }
      let(:graham) { User.find 'graham' }

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
    end

    context 'an astrological matchmaking app' do # rubocop:disable Metrics/BlockLength
      class Sign < ActiveRecord::Base
        include DFH::ActiveRecordFetcher
        self.primary_key = 'name'
        has_many :people, foreign_key: :sign_name
      end

      class Person < ActiveRecord::Base
        include DFH::ActiveRecordFetcher
        self.primary_key = 'name'
        belongs_to :sign, foreign_key: :sign_name
      end

      before do # rubocop:disable Metrics/BlockLength
        db = SQLite3::Database.new DB_FILE
        db.execute <<-SQL
          create table signs (
            name varchar(16) not null primary key,
            element varchar(8) not null,
            ruler varchar(8) not null
          );
        SQL

        db.execute <<-SQL
          create table people (
            name varchar(32) not null primary key,
            sign_name varchar(16) not null
          );
        SQL

        ActiveRecord::Base.establish_connection(
          adapter: 'sqlite3',
          database: DB_FILE
        )

        [%w[aries fire mars],
         %w[taurus earth venus],
         %w[gemini air mercury],
         %w[cancer water moon],
         %w[leo fire sun],
         %w[virgo earth mercury],
         %w[libra air venus],
         %w[scorpio water mars],
         %w[sagittarius fire jupiter],
         %w[capricorn earth saturn],
         %w[aquarius air saturn],
         %w[pisces water jupiter]].each do |name, element, ruler|
          Sign.create(name: name, element: element, ruler: ruler)
        end

        [%w[robin scorpio],
         %w[pat taurus],
         %w[dylan virgo],
         %w[terry libra],
         %w[chris aquarius],
         %w[tyler leo],
         %w[eden cancer],
         %w[dakota capricorn],
         %w[charlie aries],
         %w[alex gemini],
         %w[sam pisces],
         %w[avery sagittarius]].each do |name, sign|
          Person.create(name: name, sign_name: sign)
        end

        subject.register_class(
          Sign,
          fields: {
            name: String,
            element: String,
            ruler: String,
            people: Relation.new(
              kind: 'many',
              other_type: 'Person',
              my_field: 'name',
              other_field: 'sign_name'
            )
          }
        )

        subject.register_class(
          Person,
          fields: {
            name: String,
            sign_name: String,
            sign: Relation.new(
              kind: 'one',
              other_type: 'Sign',
              my_field: 'sign_name',
              other_field: 'name'
            )
          }
        )
      end

      it 'applies sound elemental reasoning' do
        subject.load_str <<~POL
          allow("the water of aquarius", "slake", x: Person) if
            x.sign.element in ["air", "earth", "water"];
          allow("the venom of scorpio", "intoxicate", x: Person) if
            x.sign.element in ["air", "fire"];
          allow("the venom of scorpio", "intoxicate", x: Person) if
            x.sign.ruler in ["saturn", "neptune"];
        POL

        water_winners = Person.joins(:sign).where.not(signs: { element: 'fire' })
        check_authz 'the water of aquarius', 'slake', Person, water_winners

        venom_victims =
          Person.joins(:sign).where(signs: { element: %w[air fire] })
                .or(Person.joins(:sign).where(signs: { ruler: %w[saturn neptune] }))
        check_authz 'the venom of scorpio', 'intoxicate', Person, venom_victims
      end

      it 'assigns auspicious matches' do
        # FIXME(gw) probably not astrologically correct
        subject.load_str <<~POL
          align(_: Sign{ruler: r},   _: Sign{ruler: r});
          align(_: Sign{element: e}, _: Sign{element: e});
          allow(a: Person, "match", b: Person) if
            a != b and align(a.sign, b.sign);
        POL

        compatible_signs = lambda do |sign|
          Sign.where(element: sign.element).or Sign.where(ruler: sign.ruler)
        end

        compatible_people = lambda do |person|
          Person.where.not(name: person.name).where(sign: compatible_signs[person.sign])
        end

        Person.all.each do |person|
          check_authz person, 'match', Person, compatible_people[person]
        end
      end
    end
  end
end
