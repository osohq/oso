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
    def self.included(base) # rubocop:disable Metrics/MethodLength, Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/PerceivedComplexity
      base.class_eval do # rubocop:disable Metrics/BlockLength
        it = {}
        param = lambda do |field, value|
          field.nil? ? { primary_key => value.send(primary_key) } : { field => value }
        end

        it['Eq'] = it['In'] = ->(q, f, v) { q.where param[f, v] }
        it['Neq'] = it['Nin'] = ->(q, f, v) { q.where.not param[f, v] }
        it.default_proc = proc { |k| raise "Unsupported constraint kind: #{k}" }
        it.freeze

        instance_variable_set :@constrain, it

        def self.build_query(cons) # rubocop:disable Metrics/MethodLength, Metrics/AbcSize
          cons.reduce(all) do |qu, c|
            if c.field.is_a? Array
              co =  @constrain[c.kind == 'In' ? 'Eq' : 'Neq']
              conds = c.value.map do |v|
                c.field.zip(v).reduce(qu) do |q, z|
                  co[q, *z]
                end
              end
              conds.any? ? conds.reduce(:or) : none
            else
              @constrain[c.kind][qu, c.field, c.value]
            end
          end
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
