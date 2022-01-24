# frozen_string_literal: true

module DataFilteringHelpers
  def count(coll)
    coll.reduce(Hash.new(0)) { |c, x| c.tap { c[x] += 1 } }
  end

  def self.record(*args, &blk)
    Struct.new(*args, &blk).include(AutoFetcher)
  end

  def check_new_authorized_query(actor, action, resource, expected)
    results = subject.send(:new_authorized_query, actor, action, resource).to_a
    expect(results).to contain_exactly(*expected)
    expected.each do |it|
      answer = subject.query_rule 'allow', actor, action, it
      expect(answer.to_a).not_to be_empty
    end
  end

  def check_authz(*args)
    check_new_authorized_query(*args)
  end

  module AutoFetcher
    def self.included(base)
      base.instance_variable_set :@instances, []
      def base.all
        @instances
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

RSpec.configure do |c|
  c.include DataFilteringHelpers
end

Relation = ::Oso::Relation
DFH = DataFilteringHelpers
