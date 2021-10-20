# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module Data

      class DataFilter
        def to_a
          to_query.to_a
        end
        class << self
          alias [] new
        end
      end
      # We parse filter plans into instances of these classes.
      #
      # Users subclass them and implement a `to_query` method to define
      # custom behavior (see the ActiveRecord example below)

      # Bottom level wrapper for a user model type
      class Source < DataFilter
        attr_reader :model
        def initialize(model)
          @model = model
        end
      end

      # Abstract selection from a relation
      class Select < DataFilter
        attr_reader :base, :field, :kind, :value
        def initialize(base, field, value, kind: :eq)
          @base = base
          @field = field
          @value = value
          @kind = kind
        end
      end


      # Abstract join operation between two relations
      class Join < DataFilter
        attr_reader :left, :right, :kind
        def initialize(left, right, kind: :inner)
          @left = left
          @right = right
          @kind = kind
        end
      end

      class Proj < DataFilter
        attr_reader :model, :attr
        def initialize(model, attr)
          @model = model
          @attr = attr
        end
      end

      class Value < DataFilter
        attr_reader :value
        def initialize(value)
          @value = value
        end
        def to_column
          "#{@value}"
        end
      end

      # Subclasses for ActiveRecord
      # 
      #
      class ArelSource < Source
        def to_query
          @model.all
        end
      end

      class ArelProj < Proj
        def to_column
          "#{@model.table_name}.#{@attr}"
        end
      end

      class ArelSelect < Select
        OOPS = {eq: '=', in: 'IN', nin: 'NOT IN', neq: '!='}

        def to_query
          q = base.to_query
          lhs = "#{field.to_column} #{OOPS[kind]}"
          case value
          when Proj
            q.where("#{lhs} #{value.to_column}")
          else
            q.where("#{lhs} ?", value.value)
          end
        end
      end

      class ArelJoin < Join
        def to_query
          left.model.all.joins(
            "INNER JOIN #{right.model.table_name} ON #{left.to_column} = #{right.to_column}"
          )
        end
      end
    end
  end
end
