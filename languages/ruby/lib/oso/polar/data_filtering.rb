# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module DataFiltering
      # Represents a set of filter sequences that should allow the host
      # to obtain the records satisfying a query.
      class FilterPlan
        attr_reader :result_sets

        def self.parse(polar, partials, class_name)
          types = polar.host.serialize_types
          parsed_json = polar.ffi.build_filter_plan(types, partials, 'resource', class_name)
          result_sets = parsed_json['result_sets'].map do |rset|
            ResultSet.parse polar, rset
          end

          new polar: polar, result_sets: result_sets
        end

        def initialize(polar:, result_sets:)
          @polar = polar
          @result_sets = result_sets
        end

        def build_query # rubocop:disable Metrics/MethodLength, Metrics/AbcSize
          combine = nil
          result_sets.each_with_object([]) do |rs, qb|
            rs.resolve_order.each_with_object({}) do |i, set_results|
              req = rs.requests[i]
              cs = req.constraints.each { |c| c.ground set_results }
              typ = @polar.host.types[req.class_tag]
              q = typ.build_query[cs]
              if i != rs.result_id
                set_results[i] = typ.exec_query[q]
              else
                combine = typ.combine_query
                qb.push q
              end
            end
          end.reduce(&combine)
        end

        # Represents a sequence of filters for one set of results
        class ResultSet
          attr_reader :requests, :resolve_order, :result_id

          def self.parse(polar, parsed_json)
            resolve_order = parsed_json['resolve_order']
            result_id = parsed_json['result_id']
            requests = parsed_json['requests'].each_with_object({}) do |req, reqs|
              reqs[req[0].to_i] = Request.parse(polar, req[1])
            end

            new resolve_order: resolve_order, result_id: result_id, requests: requests
          end

          def initialize(requests:, resolve_order:, result_id:)
            @resolve_order = resolve_order
            @requests = requests
            @result_id = result_id
          end
        end

        # Represents a filter for a result set
        class Request
          attr_reader :constraints, :class_tag

          def self.parse(polar, parsed_json)
            constraints = parsed_json['constraints'].map do |con|
              Constraint.parse polar, con
            end
            class_tag = parsed_json['class_tag']

            new(constraints: constraints, class_tag: class_tag)
          end

          def initialize(constraints:, class_tag:)
            @constraints = constraints
            @class_tag = class_tag
          end
        end
      end

      # Represents relationships between resources, eg. parent/child
      class Relation
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
          'Neq' => ->(a, b) { a != b },
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
          item = field.nil? ? item : item.send(field)
          @check[item, val]
        end

        def self.parse(polar, constraint) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
          kind = constraint['kind']
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

          new kind: kind, field: field, value: value
        end
      end
    end
  end
end
