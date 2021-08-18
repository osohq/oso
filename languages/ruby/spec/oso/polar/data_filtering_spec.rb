# frozen_string_literal: true

require 'tempfile'

require_relative './helpers'

Bar = Struct.new :id, :is_cool, :is_still_cool
Foo = Struct.new :id, :bar_id, :is_fooey, :numbers

Foo.class_eval do
  define_method(:bar) { BARS.find { |bar| bar.id == bar_id } }
end

FOOS = [
  Foo.new('something', 'hello', false, []),
  Foo.new('another', 'hello', true, [1]),
  Foo.new('third', 'hello', true, [2]),
  Foo.new('fourth', 'goodbye', true, [2, 1])
].freeze

BARS = [
  Bar.new('hello', true, true),
  Bar.new('goodbye', false, true),
  Bar.new('hershey', false, false)
].freeze

FETCH = ->(arr) { ->(cs) { arr.select { |x| cs.all? { |c| c.check(x) } } } }

Relationship = ::Oso::Polar::DataFiltering::Relationship

Org = Struct.new :name
Repo = Struct.new :name, :org_name
Issue = Struct.new :name, :repo_name
User = Struct.new :name
Role = Struct.new :user_name, :resource_name, :role

RSpec.configure do |c|
  c.include Helpers
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
      let(:check_authz) do
        lambda do |actor, action, resource, expected|
          results = subject.get_allowed_resources(actor, action, resource)
          expect(unord_eq(results, expected)).to be true
          expected.each do |re|
            answer = subject.allowed?(actor: actor, action: action, resource: re)
            expect(answer).to be true
          end
        end
      end

      before do # rubocop:disable Metrics/BlockLength
        subject.register_class(
          Org,
          fields: { 'name' => String },
          fetcher: FETCH[[apple, osohq]]
        )
        subject.register_class(
          Repo,
          fetcher: FETCH[[oso, ios, demo]],
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
          fetcher: FETCH[[bug, laggy]],
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
          fetcher: FETCH[[leina, steve, gabe]],
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
          fetcher: FETCH[roles],
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
          check_authz[leina, 'invite', Org, [osohq]]
          check_authz[leina, 'pull', Repo, [oso, demo]]
          check_authz[leina, 'push', Repo, [oso, demo]]
          check_authz[leina, 'edit', Issue, [bug]]
        end
      end

      context 'org members' do
        it "can only pull the org's repos" do
          check_authz[steve, 'invite', Org, []]
          check_authz[steve, 'pull', Repo, [oso, demo]]
          check_authz[steve, 'push', Repo, []]
          check_authz[steve, 'edit', Issue, []]
        end
      end

      context 'repo writers' do
        it 'can push, pull, and edit issues' do
          check_authz[gabe, 'invite', Org, []]
          check_authz[gabe, 'pull', Repo, [oso]]
          check_authz[gabe, 'push', Repo, [oso]]
          check_authz[gabe, 'edit', Issue, [bug]]
        end
      end
    end

    context 'when filtering unknown values' do # rubocop:disable Metrics/BlockLength
      before do
        subject.register_class(
          Bar,
          fetcher: FETCH[BARS],
          fields: {
            'id' => String,
            'is_cool' => PolarBoolean,
            'is_still_cool' => PolarBoolean
          }
        )

        subject.register_class(
          Foo,
          fetcher: FETCH[FOOS],
          fields: {
            'id' => String,
            'bar_id' => String,
            'is_fooey' => PolarBoolean,
            'numbers' => Array,
            'bar' => ::Oso::Polar::DataFiltering::Relationship.new(
              kind: 'parent',
              other_type: 'Bar',
              my_field: 'bar_id',
              other_field: 'id'
            )
          }
        )
      end

      context 'without relationships' do # rubocop:disable Metrics/BlockLength
        it 'works' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.is_fooey = true;'
          subject.load_str(policy)
          results = subject.get_allowed_resources('gwen', 'get', Foo)
          expected = FOOS.select(&:is_fooey)
          expect(expected).not_to be_empty
          expect(unord_eq(results, expected)).to be true
        end

        context 'the in operator' do
          it 'finds values in variables' do
            policy = 'allow("gwen", "get", foo: Foo) if 1 in foo.numbers and 2 in foo.numbers;'
            subject.load_str(policy)
            results = subject.get_allowed_resources('gwen', 'get', Foo)
            expected = FOOS.select { |f| f.numbers.include?(1) and f.numbers.include?(2) }
            expect(expected).not_to be_empty
            expect(unord_eq(results, expected)).to be true
          end
          it 'finds variables in values' do
            policy = 'allow("gwen", "eat", foo: Foo) if foo.numbers in [[1]];'
            subject.load_str(policy)
            results = subject.get_allowed_resources('gwen', 'eat', Foo)
            expected = FOOS.select { |f| f.numbers == [1] }
            expect(expected).not_to be_empty
            expect(unord_eq(results, expected)).to be true
          end
        end

        it 'can compare two fields on the same object' do
          policy = 'allow(_, _, bar: Bar) if bar.is_cool = bar.is_still_cool;'
          subject.load_str(policy)
          results = subject.get_allowed_resources('gwen', 'eat', Bar)
          expected = BARS.select { |b| b.is_cool == b.is_still_cool }
          expect(expected).not_to be_empty
          expect(unord_eq(results, expected)).to be true
        end
      end

      context 'with relationships' do
        it 'works' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.bar = bar and bar.is_cool = true and foo.is_fooey = true;'
          subject.load_str(policy)
          results = subject.get_allowed_resources('gwen', 'get', Foo)
          expected = FOOS.select { |foo| foo.bar.is_cool and foo.is_fooey }
          expect(expected).not_to be_empty
          expect(unord_eq(results, expected)).to be true
        end
      end
    end
  end
end
