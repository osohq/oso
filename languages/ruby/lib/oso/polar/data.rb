# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module Data
      # Abstract adapter supertype
      class Adapter
        def build_query(_types, _filter)
          raise "build_query not implemented for #{self}"
        end

        def execute_query(_query)
          raise "execute_query not implemented for #{self}"
        end

        # Adapter for ActiveRecord
        class ActiveRecordAdapter < Adapter
          def build_query(types, filter) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
            q = filter.relations.reduce(filter.model.all) do |q1, rel|
              rec = types[rel.left].fields[rel.name]
              q1.joins(
                "INNER JOIN #{rel.right.table_name} ON " \
                "#{rel.left.table_name}.#{rec.my_field} = " \
                "#{rel.right.table_name}.#{rec.other_field}"
              )
            end

            filter.conditions.map do |conjs|
              conjs.reduce(q) do |q1, conj|
                q1.where(*conj.to_sql_args)
              end
            end.reduce(:or).distinct
          end

          def execute_query(query)
            query.to_a
          end
        end
      end

      # Data Filter
      class Filter
        attr_reader :model, :relations, :conditions

        def initialize(model:, relations:, conditions:)
          @model = model
          @relations = relations
          @conditions = conditions
        end

        def to_query(types)
          ActiveRecordAdapter.new.build_query(types, self)
        end

        def self.parse(polar, blob)
          model = polar.host.types[blob['root']].klass.get
          relations = blob['relations'].map do |rel|
            Relation.parse(polar, *rel)
          end
          conditions = blob['conditions'].map do |disj|
            disj.map { |conj| Condition.parse(polar, *conj) }
          end
          new(model: model, relations: relations, conditions: conditions)
        end

        Projection = Struct.new(:source, :field) do
          # this depends on the ORM
          def columnize
            "#{source.table_name}.#{field || source.primary_key}"
          end
        end

        Relation = Struct.new(:left, :name, :right) do
          # this doesn't depend on the ORM
          def self.parse(polar, left, name, right)
            Relation.new(polar.name_to_class(left), name, polar.name_to_class(right))
          end
        end

        Condition = Struct.new(:left, :cmp, :right) do # rubocop:disable Metrics/BlockLength
          OPS = {
            'Eq' => '=', 'In' => 'IN', 'Nin' => 'NOT IN', 'Neq' => '!='
          }.freeze

          # this is very ORM specific
          def to_sql_args
            args = []
            lhs = self.class.to_sql_arg left, args
            rhs = self.class.to_sql_arg right, args
            args.unshift "#{lhs} #{OPS[cmp]} #{rhs}"
          end

          def self.to_sql_arg(side, args)
            side.is_a?(Projection) ? side.columnize : '?'.tap { args.push side }
          end

          def self.parse(polar, left, cmp, right)
            Condition.new(parse_side(polar, left), cmp, parse_side(polar, right))
          end

          def self.parse_side(polar, side)
            key = side.keys.first
            val = side[key]
            case key
            when 'Field'
              Projection.new(polar.name_to_class(val[0]), val[1])
            when 'Immediate'
              polar.host.to_ruby('value' => [[val.keys.first, val.values.first]])
            else
              raise key
            end
          end
        end
      end
    end
  end
end
