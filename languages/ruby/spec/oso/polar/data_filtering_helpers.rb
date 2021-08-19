# frozen_string_literal: true

module Fetch
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
end

Foo = Struct.new(:id, :bar_id, :is_fooey, :numbers) do
  include Fetch
  def bar
    Bar.all.find { |bar| bar.id == bar_id }
  end
end

def record(*args)
  Struct.new(*args).include Fetch
end

Bar = record(:id, :is_cool, :is_still_cool)

Foo.new('something', 'hello', false, [])
Foo.new('another', 'hello', true, [1])
Foo.new('third', 'hello', true, [2])
Foo.new('fourth', 'goodbye', true, [2, 1])

Bar.new('hello', true, true)
Bar.new('goodbye', false, true)
Bar.new('hershey', false, false)

Relationship = ::Oso::Polar::DataFiltering::Relationship

Org = record :name
Repo = record :name, :org_name
Issue = record :name, :repo_name
User = record :name
Role = record :user_name, :resource_name, :role

Wizard = record(:name, :books, :spell_levels)
Spell = record(:name, :school, :level)
Familiar = record(:name, :kind, :wizard_name)
