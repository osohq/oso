# frozen_string_literal: true

module Oso
  module Polar
    # Polar variable.
    module DataFiltering

      class Relationship
        attr_reader :kind, :other_type, :my_field, :other_field
        def initialize(kind:, other_type:, my_field:, other_field:)
          @kind = kind
          @other_type = other_type
          @my_field = my_field
          @other_field = other_field
        end
      end

      class Field
        attr_reader :field
        def initialize(field:)
          @field = field
        end
      end
      class Ref
        attr_reader :field, :result_id
        def initialize(field:, result_id:)
          @field = field
          @result_id = result_id
        end
      end

      class Constraint
        attr_reader :kind, :field, :value
        def initialize(kind:, field:, value:)
          @kind = kind
          @field = field
          @value = value
        end

        def to_predicate
          get_val = value.is_a?(Field) ? ->(x) { x.send value.field } : ->(_) { value }
          case kind
          when 'Eq'
            ->(x) { x.send(field) == get_val[x] }
          when 'In'
            ->(x) { get_val[x].include? x.send(field) }
          when 'Contains'
            ->(x) { x.send(field).include? get_val[x] }
          else
            raise "Unknown constraint kind `#{kind}`"
          end
        end

        def ground(results)
          if value.is_a? Ref
            ref = value
            @value = results[ref.result_id]
            @value = value.map {|v| v.send ref.field} unless ref.field.nil?
          end
        end
      end


      def self.parse_constraint(polar, constraint)
        kind = constraint['kind']
        raise unless %w(Eq In Contains).include? kind
        field = constraint['field']
        value = constraint['value']

        value_kind = value.first
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

      def self.builtin_resolve(polar, filter_plan)
        result_sets = filter_plan['result_sets']
        results = []
        result_sets.each do |rs|
          set_results = {}
          requests = rs['requests']
          resolve_order = rs['resolve_order']
          result_id = rs['result_id']

          resolve_order.each do |i|
            req = requests[i.to_s]
            class_name = req['class_tag']
            constraints = req['constraints'].map(&method(:parse_constraint))
            constrains.each {|c| con.ground set_results }
            fetcher = polar.host.types[class_name].fetcher
            set_results[i] = fetcher.call constraints
          end

          results += set_results[result_id]
        end
        results.uniq
      end

      # @param name [String]
      def self.filter(polar, filter_plan, filter_plan_resolver: method(:builtin_resolve))
        filter_plan_resolver.call(polar, filter_plan)
      end

    end
  end
end
