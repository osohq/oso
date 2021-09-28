# frozen_string_literal: true

module DataFilteringHelpers
  def count(coll)
    coll.reduce(Hash.new(0)) { |c, x| c.tap { c[x] += 1 } }
  end

  def check_authz(actor, action, resource, expected)
    results = subject.authorized_resources(actor, action, resource)
    expect(results).to contain_exactly(*expected)
    expected.each do |it|
      answer = subject.query_rule 'allow', actor, action, it
      expect(answer.to_a).not_to be_empty
    end
  end

  def self.record(*args, &blk)
    Struct.new(*args, &blk).include(AutoFetcher)
  end

  module AutoFetcher
    def self.included(base) # rubocop:disable Metrics/MethodLength
      base.instance_variable_set :@instances, []

      def base.all
        @instances
      end

      def base.combine_query(one, two)
        one + two
      end

      def base.exec_query(query)
        query.uniq
      end

      def base.build_query(cons)
        all.select { |x| cons.all? { |c| c.check x } }
      end

      class << base
        alias_method :_new, :new
        define_method :new do |*args|
          _new(*args).tap { |me| all.push me }
        end
      end
    end
  end

  module ActiveRecordFetcher
    def self.included(base) # rubocop:disable Metrics/MethodLength, Metrics/AbcSize
      base.class_eval do
        it = {}
        param = lambda do |c|
          if c.field.nil?
            { primary_key => c.value.send(primary_key) }
          else
            { c.field => c.value }
          end
        end

        it['Eq'] = it['In'] = ->(q, c) { q.where param[c] }
        it['Neq'] = ->(q, c) { q.where.not param[c] }
        it.default_proc = proc { |k| raise "Unsupported constraint kind: #{k}" }
        it.freeze

        instance_variable_set :@constrain, it

        def self.build_query(cons)
          cons.reduce(all) { |q, c| @constrain[c.kind][q, c] }
        end

        def self.exec_query(query)
          query.distinct.to_a
        end

        def self.combine_query(one, two)
          one.or(two)
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
