# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module Data

      # Data filtering configuration now consists of:
      #
      # 1. Subclass the abstract query classes and implement `to_query` for each one.
      # 2. Register the subclass implementations with the host.
      # 3. Tell us about relations so we can construct joins in the core.
      #
      # `build_filter_plan` (or w/e we end up calling it) will then parse the
      # query into filter objects. `to_query` turns a filter into a query. Steps 1
      # and 2 above replace the `build_query` etc. callbacks users had to provide
      # before.

      # Abstract superclass
      # not really needed, methods are just conveniences for demo purposes
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
        # lcol: proj
        # rcol: proj
        # right: filter
        # kind : { :inner, :outer } ( currently ignored )
        def initialize(left, lcol, rcol, right, kind: :inner)
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
      # constructing joins. We could separate the data filtering API better from the
      # rest of the host library by moving that out of register_class and into a new
      # method that also handles filter subclass registration, for example:
      #
      #   oso.configure_data_filtering(
      #     source: ArelSource,
      #     select: ArelSelect,
      #     join: ArelJoin,
      #     relations: {
      #       Issue => { repo: [Repo, :repo_id, :id] },
      #       Org => { repos: [Repo, :id, :org_id] },
      #       Repo => {
      #         org: [Org, :org_id, :id],
      #         issues: [Issues, :id, :repo_id]
      #       },
      #     }
      #   )
      #
      # Eventually we could even have per-model filter subclasses to handle different
      # data sources.
      
      class ArelSource < Source
        def to_query
          @model.all
        end
      end

      # Hack to generate unambiguous column names.
      # Ideally we could alias every join and generate all column names
      # explicitly in the core.
      module ArelColumnizer
        private
        def columnize(proj)
          "#{proj.source.to_query.table_name}.#{proj.field}"
        end
      end

      class ArelSelect < Select
        include ArelColumnizer
        OPS = {eq: '=', in: 'IN', nin: 'NOT IN', neq: '!='}

        def to_query
          query = source.to_query
          left = "#{columnize(lhs)} #{OPS[kind]}"
          case rhs
          when Proj then query.where("#{left} #{columnize(rhs)}")
          when Value then query.where("#{left} ?", rhs.value)
          else raise TypeError, rhs
          end
        end
      end

      class ArelJoin < Join
        include ArelColumnizer
        def to_query
          lhs = left.to_query
          rhs = right.to_query
          lhs.joins "INNER JOIN #{rhs.table_name} ON #{columnize(lcol)} = #{columnize(rcol)}"
        end
      end
    end
  end
end
