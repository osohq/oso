# frozen_string_literal: true

module DataFilteringHelpers
  def count(coll)
    coll.reduce(Hash.new(0)) { |c, x| c.tap { c[x] += 1 } }
  end

  def unord_eq(left, right)
    count(left) == count(right)
  end

  def fetcher(coll)
    ->(cons) { coll.select { |x| cons.all? { |c| c.check x } } }
  end

  def check_authz(actor, action, resource, expected)
    results = subject.get_allowed_resources(actor, action, resource)
    expect(unord_eq(results, expected)).to be true
    expected.each do |re|
      answer = subject.allowed?(actor: actor, action: action, resource: re)
      expect(answer).to be true
    end
  end

  module Fetcher
    def self.included(base) # rubocop:disable Metrics/MethodLength
      base.instance_variable_set :@instances, []

      def base.all
        @instances
      end

      def base.fetcher
        ->(cons) { @instances.select { |x| cons.all? { |c| c.check x } } }
      end

      class << base
        alias_method :_new, :new
        define_method :new do |*args|
          _new(*args).tap { |me| all.push me }
        end
      end
    end
  end
end

Relationship = ::Oso::Polar::DataFiltering::Relationship

def record(*args, &blk)
  Struct.new(*args, &blk).include(DataFilteringHelpers::Fetcher)
end

Bar = record(:id, :is_cool, :is_still_cool)
Foo = record(:id, :bar_id, :is_fooey, :numbers) do
  def bar
    Bar.all.find { |bar| bar.id == bar_id }
  end
end
FooLog = record(:id, :foo_id, :data)

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

Org = record :name
Repo = record :name, :org_name
Issue = record :name, :repo_name
User = record :name
Role = record :user_name, :resource_name, :role

Wizard = record(:name, :books, :spell_levels) do
  def spells
    Spell.all.select do |spell|
      books.include?(spell.school) and spell_levels.include?(spell.level)
    end
  end
end

Familiar = record :name, :kind, :wizard_name
Spell = record :name, :school, :level
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
