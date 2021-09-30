# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module DataFiltering
      GETATTR = ->(x, attr) { attr.nil? ? x : x.send(attr) }
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
              cs = req.ground(set_results)
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
            @polar = polar
            constraints = parsed_json['constraints'].map do |con|
              Filter.parse polar, con
            end
            class_tag = parsed_json['class_tag']

            new(constraints: constraints, class_tag: class_tag)
          end

          def ground(results) # rubocop:disable Metrics/MethodLength, Metrics/CyclomaticComplexity, Metrics/AbcSize, Metrics/PerceivedComplexity
            xrefs, rest = constraints.partition do |c|
              c.value.is_a?(Ref) and !c.value.result_id.nil?
            end

            yrefs, nrefs = xrefs.partition { |r| %w[In Eq].include? r.kind }
            [[yrefs, 'In'], [nrefs, 'Nin']].each do |refs, kind|
              next unless refs.any?

              refs.group_by { |f| f.value.result_id }.each do |rid, fils|
                value = results[rid].map { |r| fils.map { |f| GETATTR[r, f.value.field] } }
                field = fils.map(&:field)
                rest.push(Filter.new(kind: kind, value: value, field: field))
              end
            end
            rest
          end

          def initialize(constraints:, class_tag:)
            @constraints = constraints
            @class_tag = class_tag
          end
        end
      end

      # Represents relationships between resources, eg. one-one or one-many
      class Relation
        attr_reader :kind, :other_type, :my_field, :other_field

        # Describe a Relation from one type to another.
        # @param kind [String] The type of relation, either "one" or "many"
        # @param other_type The name or class object of the related type
        # @param my_field The field on this type that matches +other_type+
        # @param other_field The field on +other_type+ that matches this type
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
      class Filter
        attr_reader :kind, :field, :value

        CHECKS = {
          'Eq' => ->(a, b) { a == b },
          'In' => ->(a, b) { b.include? a },
          'Neq' => ->(a, b) { a != b },
          'Nin' => ->(a, b) { !b.include?(a) },
          'Contains' => ->(a, b) { a.include? b }
        }.freeze

        # Create a new predicate for data filtering.
        # @param kind [String] Represents a condition. One of "Eq", "Neq", "In", "Contains".
        # @param field The field the condition applies to.
        # @param value The value with which to compare the field according to the condition.
        def initialize(kind:, field:, value:)
          @kind = kind
          @field = field
          @value = value
        end

        def ground(results)
          return unless value.is_a? Ref

          ref = value
          @value = results[ref.result_id]
          @value = value.map { |v| v.send ref.field } unless ref.field.nil?
        end

        def check(item) # rubocop:disable Metrics/AbcSize
          val = value.is_a?(Field) ? item.send(value.field) : value
          item = if field.nil?
                   item
                 elsif field.is_a? Array
                   field.map { |f| GETATTR[item, f] }
                 else
                   item.send field
                 end
          CHECKS[@kind][item, val]
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
