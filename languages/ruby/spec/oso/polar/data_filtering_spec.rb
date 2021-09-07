# frozen_string_literal: true

require 'tempfile'

require_relative './helpers'
require_relative './data_filtering_helpers'

RSpec.configure do |c|
  c.include DataFilteringHelpers
end

RSpec.describe Oso::Polar::Polar do # rubocop:disable Metrics/BlockLength
  DFH = DataFilteringHelpers
  Relation = ::Oso::Polar::DataFiltering::Relation
  Bar = DFH.record(:id, :is_cool, :is_still_cool)
  Foo = DFH.record(:id, :bar_id, :is_fooey, :numbers) do
    def bar
      Bar.all.find { |bar| bar.id == bar_id }
    end
  end
  FooLog = DFH.record(:id, :foo_id, :data)

  Foo.new('something', 'hello', false, [])
  Foo.new('another', 'hello', true, [1])
  Foo.new('third', 'hello', true, [2])
  Foo.new('fourth', 'goodbye', true, [2, 1])

  Bar.new('hello', true, true)
  Bar.new('goodbye', false, true)
  Bar.new('hershey', false, false)

  FooLog.new('a', 'fourth', 'hello')
  FooLog.new('b', 'third', 'world')
  FooLog.new('c', 'another', 'steve')

  Widget = DFH.record :id
  0.upto(9).each { |id| Widget.new id }

  context '#authorized_resources' do # rubocop:disable Metrics/BlockLength
    it 'handles classes with explicit names' do
      subject.register_class(
        Widget,
        name: 'Doohickey',
        fields: { id: Integer }
      )

      subject.load_str 'allow("gwen", "eat", it: Doohickey) if it.id = 8;'
      check_authz 'gwen', 'eat', Widget, [Widget.all[8]]
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
            is_still_cool: PolarBoolean
          }
        )

        subject.register_class(
          FooLog,
          fields: {
            'id' => String,
            'foo_id' => String,
            'data' => String,
            'foo' => Relation.new(
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
              other_type: 'FooLog',
              my_field: 'id',
              other_field: 'foo_id'
            )
          }
        )
      end

      it 'can compare a field with a known value' do
        policy = 'allow("gwen", "get", foo: Foo) if foo.is_fooey = true;'
        subject.load_str(policy)
        results = subject.authorized_resources('gwen', 'get', Foo)
        expected = Foo.all.select(&:is_fooey)
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'can check if a value is in a field' do
        policy = 'allow("gwen", "get", foo: Foo) if 1 in foo.numbers and 2 in foo.numbers;'
        subject.load_str(policy)
        results = subject.authorized_resources('gwen', 'get', Foo)
        expected = Foo.all.select { |f| f.numbers.include?(1) and f.numbers.include?(2) }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'can check if a field is in a value' do
        policy = 'allow("gwen", "eat", foo: Foo) if foo.numbers in [[1]];'
        subject.load_str(policy)
        results = subject.authorized_resources('gwen', 'eat', Foo)
        expected = Foo.all.select { |f| f.numbers == [1] }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'can compare two fields on the same object' do
        policy = 'allow(_, _, bar: Bar) if bar.is_cool = bar.is_still_cool;'
        subject.load_str(policy)
        results = subject.authorized_resources('gwen', 'eat', Bar)
        expected = Bar.all.select { |b| b.is_cool == b.is_still_cool }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'can check that two fields are not equal' do
        policy = 'allow(_, _, bar: Bar) if bar.is_cool != bar.is_still_cool;'
        subject.load_str(policy)
        results = subject.authorized_resources('gwen', 'eat', Bar)
        expected = Bar.all.reject { |b| b.is_cool == b.is_still_cool }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'handles parent relationships' do
        policy = 'allow("gwen", "get", foo: Foo) if foo.bar = bar and bar.is_cool = true and foo.is_fooey = true;'
        subject.load_str(policy)
        results = subject.authorized_resources('gwen', 'get', Foo)
        expected = Foo.all.select { |foo| foo.bar.is_cool and foo.is_fooey }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'handles child relationships' do
        policy = 'allow("gwen", "get", foo: Foo) if log in foo.logs and log.data = "hello";'
        subject.load_str policy
        expected = Foo.all.select { |foo| foo.id == 'fourth' }
        check_authz 'gwen', 'get', Foo, expected
      end

      it 'handles all the relationships at once' do
        policy = <<~POL
          allow(log: FooLog, "a", foo: Foo) if log in foo.logs;
          allow(log: FooLog, "b", foo: Foo) if foo = log.foo;
          allow(log: FooLog, "c", foo: Foo) if log.foo = foo and log in foo.logs;
          allow(log: FooLog, "d", foo: Foo) if log in foo.logs and log.foo = foo;
        POL
        subject.load_str policy
        log = FooLog.all.find { |l| l.foo_id == 'fourth' }
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

      subject.load_file policy_file
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

  context 'when using Oso roles' do # rubocop:disable Metrics/BlockLength
    Org = DFH.record :name
    Repo = DFH.record :name, :org_name
    Issue = DFH.record :name, :repo_name
    User = DFH.record :name
    Role = DFH.record :user_name, :resource_name, :role
    let(:roles_file) { File.join(__dir__, 'data_filtering_roles_policy.polar') }
    let(:osohq) { Org.new('osohq') }
    let(:apple) { Org.new('apple') }
    let(:oso) { Repo.new('oso', 'osohq') }
    let(:demo) { Repo.new('demo', 'osohq') }
    let(:ios) { Repo.new('ios', 'apple') }
    let(:bug) { Issue.new('bug', 'oso') }
    let(:laggy) { Issue.new('laggy', 'ios') }
    let(:leina) { User.new('leina') }
    let(:steve) { User.new('steve') }
    let(:gabe) { User.new('gabe') }
    Role.new('leina', 'osohq', 'owner')
    Role.new('steve', 'osohq', 'member')
    Role.new('gabe', 'oso', 'writer')

    before do # rubocop:disable Metrics/BlockLength
      subject.register_class(
        Org,
        fields: { name: String }
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
      subject.register_class(
        User,
        fields: {
          name: String,
          roles: Relation.new(
            kind: 'many',
            other_type: 'Role',
            my_field: 'name',
            other_field: 'user_name'
          )
        }
      )
      subject.register_class(
        Role,
        fields: {
          user_name: String,
          resource_name: String,
          role: String
        }
      )

      subject.load_file(roles_file)
    end

    context 'org owners' do
      it 'can do anything in their org' do
        check_authz leina, 'invite', Org, [osohq]
        check_authz leina, 'pull', Repo, [oso, demo]
        check_authz leina, 'push', Repo, [oso, demo]
        check_authz leina, 'edit', Issue, [bug]
      end
    end

    context 'org members' do
      it "can only pull the org's repos" do
        check_authz steve, 'invite', Org, []
        check_authz steve, 'pull', Repo, [oso, demo]
        check_authz steve, 'push', Repo, []
        check_authz steve, 'edit', Issue, []
      end
    end

    context 'repo writers' do
      it 'can push, pull, and edit issues' do
        check_authz gabe, 'invite', Org, []
        check_authz gabe, 'pull', Repo, [oso]
        check_authz gabe, 'push', Repo, [oso]
        check_authz gabe, 'edit', Issue, [bug]
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
      module GitClub
        class User < ActiveRecord::Base
          include DFH::ActiveRecordFetcher
          self.primary_key = :name
          belongs_to :org, foreign_key: :org_name
        end
        class Repo < ActiveRecord::Base
          include DFH::ActiveRecordFetcher
          self.primary_key = :name
          belongs_to :org, foreign_key: :org_name
          has_many :issues, foreign_key: :repo_name
        end
        class Org < ActiveRecord::Base
          include DFH::ActiveRecordFetcher
          self.primary_key = :name
          has_many :users, foreign_key: :org_name
          has_many :repos, foreign_key: :org_name
        end
        class Issue < ActiveRecord::Base
          include DFH::ActiveRecordFetcher
          self.primary_key = :name
          belongs_to :repo, foreign_key: :repo_name
        end
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

        ActiveRecord::Base.establish_connection(
          adapter: 'sqlite3',
          database: DB_FILE
        )

        # create orgs
        %w[apple osohq].each do |name|
          GitClub::Org.create name: name
        end

        # create repos
        [%w[oso osohq],
         %w[demo osohq],
         %w[ios apple]].each do |name, org|
          GitClub::Repo.create name: name, org_name: org
        end

        # create users
        [%w[steve osohq],
         %w[leina osohq],
         %w[gabe osohq],
         %w[graham apple]].each do |name, org|
          GitClub::User.create name: name, org_name: org
        end

        # create issues
        [%w[bug oso],
         %w[laggy ios]].each do |name, repo|
          GitClub::Issue.create name: name, repo_name: repo
        end

        subject.register_class(
          GitClub::User,
          name: 'User',
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
          GitClub::Org,
          name: 'Org',
          fields: {
            name: String,
            users: Relation.new(
              kind: 'many',
              other_type: 'User',
              my_field: 'name',
              other_field: 'org_name'
            ),
            'repos' => Relation.new(
              kind: 'many',
              other_type: 'Repo',
              my_field: 'name',
              other_field: 'org_name'
            )
          }
        )
        subject.register_class(
          GitClub::Repo,
          name: 'Repo',
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
          GitClub::Issue,
          name: 'Issue',
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
      end

      it 'works' do
        subject.load_str <<~POL
          allow(user: User, "push", repo: Repo) if
            user.org = repo.org;
          allow(user: User, "edit", issue: Issue) if
            allow(user, "push", issue.repo);
        POL

        steve = GitClub::User.find 'steve'
        bug = GitClub::Issue.find 'bug'
        oso = GitClub::Repo.find 'oso'
        demo = GitClub::Repo.find 'demo'
        check_authz steve, 'edit', GitClub::Issue, [bug]
        check_authz steve, 'push', GitClub::Repo, [oso, demo]
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
          allow(a: Sign, "match", b: Sign) if a.element = b.element;
          allow(a: Sign, "match", b: Sign) if a.ruler = b.ruler;
          allow(a: Person, "match", b: Person) if allow(a.sign, "match", b.sign) and a != b;
        POL

        compatible_signs = lambda do |sign|
          Sign.where(element: sign.element).or Sign.where(ruler: sign.ruler)
        end

        Sign.all.each do |sign|
          check_authz sign, 'match', Sign, compatible_signs[sign]
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
