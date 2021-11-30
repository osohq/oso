# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module Data
      # Abstract data filter used by the Adapter API
      class Filter
        attr_reader :model, :relations, :conditions

        def initialize(model:, relations:, conditions:)
          @model = model
          @relations = relations
          @conditions = conditions
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

        Projection = Struct.new(:source, :field)

        Relation = Struct.new(:left, :name, :right) do
          def self.parse(polar, left, name, right)
            Relation.new(polar.name_to_class(left), name, polar.name_to_class(right))
          end
        end

        Condition = Struct.new(:left, :cmp, :right) do
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
