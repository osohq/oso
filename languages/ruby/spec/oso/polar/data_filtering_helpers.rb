# frozen_string_literal: true

module DataFilteringHelpers
  def count(coll)
    coll.reduce(Hash.new(0)) { |c, x| c.tap { c[x] += 1 } }
  end

  def unord_eq(left, right)
    count(left) == count(right)
  end

  def generic_fetcher(coll)
    ->(cons) { coll.select { |x| cons.all? { |c| c.check x } } }
  end

  def check_authz(actor, action, resource, expected)
    results = subject.get_allowed_resources(actor, action, resource)
    res = unord_eq(results, expected)
    expect(res).to be true
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

      base.const_set(:FETCHER, lambda do |cons|
        base.instance_variable_get(:@instances).select do |x|
          cons.all? { |c| c.check x }
        end
      end)

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
        kinds = Hash.new { |k| raise "Unsupported constraint kind: #{k}" }
        kinds['Eq'] = kinds['In'] = lambda do |q, c|
          q.where(
            if c.field.nil?
              { primary_key => c.value.send(primary_key) }
            else
              { c.field => c.value }
            end
          )
        end

        const_set(:FETCHER, lambda do |cons|
          cons.reduce(self) { |q, con| kinds[con.kind][q, con] }
        end)
      end
    end
  end
end
