# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module Data

      # Data filtering configuration now consists of:
      #
      # - Subclass the abstract query classes and implement `to_query` for each one
      # - Register the class implementations with the host.
      #
      # `build_filter_plan` (or w/e we end up calling it) will then parse the
      # query into filter objects. A filter object can be turned into a query using
      # `to_query` or an array using `to_a`

      # We parse filter plans into instances of subclasses of these classes.
      #
      # Abstract superclass
      class DataFilter
        def to_a
          to_query.to_a
        end
        class << self
          alias [] new
        end
      end

      # Bottom level wrapper for a user model type
      class Source < DataFilter
        attr_reader :model
        def initialize(model)
          @model = model
        end
      end

      # Abstract selection from a relation
      class Select < DataFilter
        attr_reader :source, :lhs, :rhs, :kind
        # base : filter
        # lhs : proj
        # rhs : proj | value
        # kind : { :eq, :neq, :in, :nin }
        def initialize(source, lhs, rhs, kind: :eq)
          @source = source
          @lhs = lhs
          @rhs = rhs
          @kind = kind
        end
      end


      # Abstract join operation between two relations
      # This & select have some more evolution to do.
      # We need a better way of naming joined tables,
      # right now the assumption is each table will appear
      # at most once in a query. But we could generate
      # aliases in the core.
      class Join < DataFilter
        attr_reader :left, :right, :rcol, :lcol, :kind
        # left: filter
        # lcol: symbol | string
        # rcol: symbol | string
        # right: filter
        # kind : { :inner, :outer }
        def initialize(left, lcol, right, rcol, kind: :inner)
          @left = left
          @lcol = lcol
          @rcol = rcol
          @right = right
          @kind = kind
        end
      end

      # Usually the user won't need to subclass this
      class Proj < DataFilter
        attr_reader :source, :field
        # source: filter
        # field: symbol
        def initialize(source, field)
          @source = source
          @field = field
        end

        def to_query
          source.to_query
        end
      end

      # likewise with an immediate value
      class Value < DataFilter
        attr_reader :value
        def initialize(value)
          @value = value
        end
      end

      # Subclasses for ActiveRecord
      #
      # Normally this is what the user would provide. These classes would be used
      # to deserialize the filter plan from the core. Calling `to_query` on the outermost
      # instance will then produce a single authorized query for the user. This happens
      # without user involvement, they just need to tell the host about their filter
      # subclasses so we know how to deserialize each filter (eg. different subclasses
      # for types that come from different data sources)
      #
      # The other configuration we need from the user is relational information, for
      # constructing joins
      class ArelSource < Source
        def to_query
          @model.all
        end
      end

      class ArelSelect < Select
        OPS = {eq: '=', in: 'IN', nin: 'NOT IN', neq: '!='}

        def to_query
          query = source.to_query
          left = "#{lhs.to_query.table_name}.#{lhs.field} #{OPS[kind]}"
          case rhs
          when Proj
            query.where("#{left} #{rhs.to_query.table_name}.#{rhs.field}")
          when Value
            query.where("#{left} ?", rhs.value)
          else
            raise TypeError, rhs
          end
        end
      end

      class ArelJoin < Join
        def to_query
          lhs = left.to_query
          rhs = right.to_query
          lhs.joins(
            "INNER JOIN #{rhs.table_name} ON " +
            "#{columnize(lhs, lcol)} = #{columnize(rhs, rcol)}"
          )
        end

        private
        # Kind of a hack to generate unambiguous column names. If it's a
        # string, don't mess with it, it's already exact.
        # If it's a symbol it's relative to the table, so generate the name
        # from the table & column.
        # We may just be able to alias every join and generate all
        # the column names explicitly in the core.
        def columnize(tbl, col)
          case col
          when String
            col
          when Symbol
            "#{tbl.table_name}.#{col}"
          else
            raise TypeError, col
          end
        end

        # ... & that's all you have to write in the host.
      end
    end
  end
end
