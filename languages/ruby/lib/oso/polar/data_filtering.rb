# frozen_string_literal: true

module Oso
  module Polar
    # Polar variable.
    module DataFiltering
      # Represents relationships between resources, eg. parent/child
      class Relationship
        attr_reader :kind, :other_type, :my_field, :other_field

        def initialize(kind:, other_type:, my_field:, other_field:)
          @kind = kind
          @other_type = other_type
          @my_field = my_field
          @other_field = other_field
        end
      end

      # Represents field-field relationships on one resource.
      class Field
        attr_reader :field

        def initialize(field:)
          @field = field
        end
      end

      # Represents field-field relationships on different resources.
      class Ref
        attr_reader :field, :result_id

        def initialize(field:, result_id:)
          @field = field
          @result_id = result_id
        end
      end

      # Represents a condition that must hold on a resource.
      class Constraint
        attr_reader :kind, :field, :value

        CHECKS = {
          'Eq' => ->(a, b) { a == b },
          'In' => ->(a, b) { b.include? a },
          'Contains' => ->(a, b) { a.include? b }
        }.freeze

        def initialize(kind:, field:, value:)
          @kind = kind
          @field = field
          @value = value
          @check = CHECKS[kind]
          raise "Unknown constraint kind `#{kind}`" if @check.nil?
        end

        def ground(results)
          return unless value.is_a? Ref

          ref = value
          @value = results[ref.result_id]
          @value = value.map { |v| v.send ref.field } unless ref.field.nil?
        end

        def check(item)
          val = value.is_a?(Field) ? item.send(value.field) : value
          item = item.send field
          @check[item, val]
        end
      end

      def self.parse_constraint(polar, constraint) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        kind = constraint['kind']
        raise unless %w[Eq In Contains].include? kind

        field = constraint['field']
        value = constraint['value']

        value_kind = value.keys.first
        value = value[value_kind]

        case value_kind
        when 'Term'
          value = polar.host.to_ruby value
        when 'Ref'
          child_field = value['field']
          result_id = value['result_id']
          value = Ref.new field: child_field, result_id: result_id
        when 'Field'
          value = Field.new field: value
        else
          raise "Unknown value kind `#{value_kind}`"
        end

        Constraint.new kind: kind, field: field, value: value
      end

      def self.builtin_resolve(polar, filter_plan) # rubocop:disable Metrics/AbcSize
        filter_plan['result_sets'].reduce([]) do |acc, rs|
          requests = rs['requests']
          acc + rs['resolve_order'].each_with_object({}) do |i, set_results|
            req = requests[i.to_s]
            constraints = req['constraints'].map { |con| parse_constraint(polar, con) }
            constraints.each { |c| c.ground set_results }
            set_results[i] = polar.host.types[req['class_tag']].fetcher[constraints]
          end[rs['result_id']]
        end.uniq
      end

      # @param name [String]
      def self.filter(polar, filter_plan, filter_plan_resolver: method(:builtin_resolve))
        filter_plan_resolver.call(polar, filter_plan)
      end
    end
  end
end
