# frozen_string_literal: true

module Oso
  module Polar
    module Data
      class Adapter
        # Example data filtering adapter for ActiveRecord
        class ActiveRecordAdapter < Adapter
          def build_query(filter) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
            types = filter.types

            query = filter.relations.reduce(filter.model.all) do |q, rel|
              rec = types[rel.left].fields[rel.name]
              join_type = rec.kind == 'one' ? 'LEFT' : 'INNER'
              q.joins(
                "#{join_type} JOIN #{rel.right.table_name} AS #{table_alias(rel.right, filter.model)} ON " \
                "#{table_alias(rel.left, filter.model)}.#{rec.my_field} = " \
                "#{table_alias(rel.right, filter.model)}.#{rec.other_field}"
              )
            end

            filter.conditions.map do |conds|
              conds.reduce(query) do |inner_query, cond|
                inner_query.where(*sqlize(cond, filter.model))
              end
            end.reduce(:or).distinct
          end

          def execute_query(query)
            query.to_a
          end

          OPS = {
            'Eq' => '=', 'In' => 'IN', 'Nin' => 'NOT IN', 'Neq' => '!=',
            'Lt' => '<', 'Gt' => '>', 'Leq' => '<=', 'Geq' => '>='
          }.freeze

          private

          def sqlize(cond, base_model)
            parse_condition_sides(cond_as_enum(cond), base_model)
          end

          def parse_condition_sides(cond, base_model)
            args = []
            lhs = add_side(cond.left, base_model, args)
            rhs = add_side(cond.right, base_model, args)
            args.unshift("#{lhs} #{OPS[cond.cmp]} #{rhs}")
          end

          def cond_as_enum(cond)
            projection_key, value_key = if cond.left.is_a?(::Oso::Polar::Data::Filter::Projection)
              [:left, :right]
            else
              [:right, :left]
            end

            projection = cond[projection_key]
            if projection.is_a?(::Oso::Polar::Data::Filter::Projection) && projection.field.present?

              if projection.field.end_with?("?") &&
                 projection.source.defined_enums?
                # unpacking enums shortcut methods
                enum = projection.source.defined_enums.find { |_k, v| v.key?(projection.field.chomp("?")) }
                if enum.present?
                  cond[value_key] = enum[1][projection.field.chomp("?")]
                  projection.field = enum[0]
                end
              elsif projection.source.defined_enums[projection.field]&.fetch(cond[value_key], nil)&.present?
                # unify enums
                cond[value_key] = projection.source.defined_enums[projection.field][cond[value_key]]
              end
            end

            cond
          end

          def add_side(side, base_model, args)
            if side.is_a?(::Oso::Polar::Data::Filter::Projection)
              "#{table_alias(side.source, base_model)}.#{side.field || side.source.primary_key}"
            else
              args.push(side)
              "?"
            end
          end

          def table_alias(klass, base_model)
            base_model == klass ? klass.table_name : klass.name.underscore
          end
        end
      end
    end
  end
end