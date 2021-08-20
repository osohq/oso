# frozen_string_literal: true

require 'tempfile'

require_relative './helpers'
require_relative './data_filtering_helpers'

RSpec.configure do |c|
  c.include DataFilteringHelpers
end

RSpec.describe Oso::Polar::Polar do # rubocop:disable Metrics/BlockLength
  context 'data filtering' do # rubocop:disable Metrics/BlockLength
    context 'when filtering known values' do
      it 'works' do
        subject.load_str('allow(_, _, i) if i in [1, 2];')
        subject.load_str('allow(_, _, i) if i = {};')
        expect(subject.get_allowed_resources('gwen', 'get', Integer)).to eq([1, 2])
        expect(subject.get_allowed_resources('gwen', 'get', Hash)).to eq([{}])
      end
    end
    context 'when filtering unknown values' do # rubocop:disable Metrics/BlockLength
      before do # rubocop:disable Metrics/BlockLength
        subject.register_class(
          Bar,
          fetcher: Bar.fetcher,
          fields: {
            'id' => String,
            'is_cool' => PolarBoolean,
            'is_still_cool' => PolarBoolean
          }
        )

        subject.register_class(
          FooLog,
          fetcher: FooLog.fetcher,
          fields: {
            'id' => String,
            'foo_id' => String,
            'data' => String,
            'foo' => Relationship.new(
              kind: 'parent',
              other_type: 'Foo',
              my_field: 'foo_id',
              other_field: 'id'
            )
          }
        )

        subject.register_class(
          Foo,
          fetcher: Foo.fetcher,
          fields: {
            'id' => String,
            'bar_id' => String,
            'is_fooey' => PolarBoolean,
            'numbers' => Array,
            'bar' => Relationship.new(
              kind: 'parent',
              other_type: 'Bar',
              my_field: 'bar_id',
              other_field: 'id'
            ),
            'logs' => Relationship.new(
              kind: 'children',
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
        results = subject.get_allowed_resources('gwen', 'get', Foo)
        expected = Foo.all.select(&:is_fooey)
        expect(expected).not_to be_empty
        expect(unord_eq(results, expected)).to be true
      end

      it 'can check if a value is in a field' do
        policy = 'allow("gwen", "get", foo: Foo) if 1 in foo.numbers and 2 in foo.numbers;'
        subject.load_str(policy)
        results = subject.get_allowed_resources('gwen', 'get', Foo)
        expected = Foo.all.select { |f| f.numbers.include?(1) and f.numbers.include?(2) }
        expect(expected).not_to be_empty
        expect(unord_eq(results, expected)).to be true
      end

      it 'can check if a field is in a value' do
        policy = 'allow("gwen", "eat", foo: Foo) if foo.numbers in [[1]];'
        subject.load_str(policy)
        results = subject.get_allowed_resources('gwen', 'eat', Foo)
        expected = Foo.all.select { |f| f.numbers == [1] }
        expect(expected).not_to be_empty
        expect(unord_eq(results, expected)).to be true
      end

      it 'can compare two fields on the same object' do
        policy = 'allow(_, _, bar: Bar) if bar.is_cool = bar.is_still_cool;'
        subject.load_str(policy)
        results = subject.get_allowed_resources('gwen', 'eat', Bar)
        expected = Bar.all.select { |b| b.is_cool == b.is_still_cool }
        expect(expected).not_to be_empty
        expect(unord_eq(results, expected)).to be true
      end

      it 'handles parent relationships' do
        policy = 'allow("gwen", "get", foo: Foo) if foo.bar = bar and bar.is_cool = true and foo.is_fooey = true;'
        subject.load_str(policy)
        results = subject.get_allowed_resources('gwen', 'get', Foo)
        expected = Foo.all.select { |foo| foo.bar.is_cool and foo.is_fooey }
        expect(expected).not_to be_empty
        expect(unord_eq(results, expected)).to be true
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
        %w[a b c d].each do |x|
          check_authz log, x, Foo, foos
        end
      end
    end

    context 'when meddling in the affairs of wizards' do # rubocop:disable Metrics/BlockLength
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
          fetcher: Wizard.fetcher,
          fields: {
            'name' => String,
            'books' => Array,
            'spell_levels' => Array,
            'familiars' => Relationship.new(
              kind: 'children',
              other_type: 'Familiar',
              my_field: 'name',
              other_field: 'wizard_name'
            )
          }
        )

        subject.register_class(
          Spell,
          fetcher: Spell.fetcher,
          fields: {
            'name' => String,
            'school' => String,
            'level' => Integer
          }
        )

        subject.register_class(
          Familiar,
          fetcher: Familiar.fetcher,
          fields: {
            'name' => String,
            'kind' => String,
            'wizard_name' => String,
            'wizard' => Relationship.new(
              kind: 'parent',
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
      let(:roles) do
        [Role.new('leina', 'osohq', 'owner'),
         Role.new('steve', 'osohq', 'member'),
         Role.new('gabe', 'oso', 'writer')]
      end

      before do # rubocop:disable Metrics/BlockLength
        subject.register_class(
          Org,
          fields: { 'name' => String },
          fetcher: fetcher([apple, osohq])
        )
        subject.register_class(
          Repo,
          fetcher: fetcher([oso, ios, demo]),
          fields: {
            'name' => String,
            'org_name' => String,
            'org' => Relationship.new(
              kind: 'parent',
              other_type: 'Org',
              my_field: 'org_name',
              other_field: 'name'
            )
          }
        )
        subject.register_class(
          Issue,
          fetcher: Issue.fetcher,
          fields: {
            'name' => String,
            'repo_name' => String,
            'repo' => Relationship.new(
              kind: 'parent',
              other_type: 'Repo',
              my_field: 'repo_name',
              other_field: 'name'
            )
          }
        )
        subject.register_class(
          User,
          fetcher: User.fetcher,
          fields: {
            'name' => String,
            'roles' => Relationship.new(
              kind: 'children',
              other_type: 'Role',
              my_field: 'name',
              other_field: 'user_name'
            )
          }
        )
        subject.register_class(
          Role,
          fetcher: fetcher(roles),
          fields: {
            'user_name' => String,
            'resource_name' => String,
            'role' => String
          }
        )

        subject.load_file(roles_file)
        subject.enable_roles
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
  end
end
