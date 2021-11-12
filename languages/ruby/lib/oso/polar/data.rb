# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module Data

      class Filter
        attr_reader :model, :relations, :conditions

        def initialize(model:, relations:, conditions:)
          @model = model
          @relations = relations
          @conditions = conditions
        end

        def to_query(types)
          q = @relations.reduce(@model.all) do |q, rel|
            rec = types[rel.left].fields[rel.name]
            q.joins(
              "INNER JOIN #{rel.right.table_name} ON " +
              "#{rel.left.table_name}.#{rec.my_field} = " +
              "#{rel.right.table_name}.#{rec.other_field}"
            )
          end

          @conditions.map do |conjs|
            conjs.reduce(q) do |q, conj|
              q.where(*conj.to_sql_args)
            end
          end.reduce(:or).distinct
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
          def columnize
            "#{source.table_name}.#{field ? field : source.primary_key}"
          end
        end

        Relation = Struct.new(:left, :name, :right) do
          def self.parse(polar, left, name, right)
            new(polar.get_class(left), name, polar.get_class(right))
          end
        end
        Condition = Struct.new(:left, :cmp, :right) do
          OPS = {
            'Eq' => '=', 'In' => 'IN', 'Nin' => 'NOT IN', 'Neq' => '!=',
          }

          def to_sql_args
            args = []
            do_side = ->(side) do
              case side
              when Projection
                side.columnize
              else
                args.push side
                "?"
              end
            end

            lhs = do_side[left]
            rhs = do_side[right]

            args.unshift "#{lhs} #{OPS[cmp]} #{rhs}"
          end

          def self.parse(polar, left, cmp, right)
            new(parse_side(polar, left), cmp, parse_side(polar, right))
          end

          def self.parse_side(polar, side)
            key = side.keys.first
            val = side[key]
            case key
            when 'Field'
              Projection.new(polar.get_class(val[0]), val[1])
            when 'Imm'
              polar.host.to_ruby(
                'value' => [[val.keys.first, val.values.first]]
              )
            else
              raise key
            end
          end
        end
      end
    end
  end
end
