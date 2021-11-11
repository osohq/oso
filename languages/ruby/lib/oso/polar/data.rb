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
        PARSERS = {
          'Select' => ->(p, j) do
            CLASSES[:select].new(
              parse(p, j['source']), 
              parse(p, j['left']),
              parse(p, j['right']),
              kind: j['kind'] || 'Eq'
            )
          end,
          'Source' => -> (p, j) do
            CLASSES[:source].new(
              p.host.types[j].klass.get
            )
          end,
          'Join' => ->(p, j) do
            parse_field = PARSERS['Field']
            CLASSES[:join].new(
              parse(p, j['left']),
              parse_field[p, j['lcol']],
              parse_field[p, j['rcol']],
            )
          end,
          'Union' => ->(p, j) do
            CLASSES[:union].new(parse(p, j['left']), parse(p, j['right']))
          end,
          'Imm' => -> (p, j) do
            CLASSES[:value]
            Value.new(p.host.to_ruby({
              'value' => [[j.keys.first, j.values.first]]
            }))
          end,
          'Field' => -> (p, j) do
            src = CLASSES[:source].new(p.host.types[j[0]].klass.get)
            CLASSES[:field].new(src, j[1])
          end
        }

        def to_a
          to_query.to_a
        end

        def to_query
          raise "`to_query` not implemented for #{self}"
        end

        class << self
          alias [] new
          def parse(polar, json)
            key = json.keys.first
            PARSERS[key][polar, json[key]]
          end
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
        def initialize(source, lhs, rhs, kind: 'Eq')
          @source = source
          @lhs = lhs
          @rhs = rhs
          @kind = kind
        end
      end

      # Abstract union / or between two filters
      class Union < DataFilter
        attr_reader :left, :right
        def initialize(left, right)
          @left = left
          @right = right
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
        def initialize(left, lcol, rcol, kind: :inner)
          @left = left
          @lcol = lcol
          @rcol = rcol
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
          @source.to_query
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

      class ArelUnion < Union
        def to_query
          left, right = @left.to_query, @right.to_query
          left.or right
        rescue ArgumentError
          (left.joins_values + right.joins_values).each do |j|
            left = extend_joins left, j
            right = extend_joins right, j
          end
          left.or right
        end

        private
        def extend_joins(q, j)
          q.joins_values.include?(j) ? q : q.joins(j)
        end
      end

      # Hack to generate unambiguous column names.
      # Ideally we could alias every join and generate all column names
      # explicitly in the core.
      module ArelColumnizer
        private
        def columnize(proj)
          return "?" if proj.is_a? Value
          query = proj.to_query
          field = proj.field || query.primary_key
          "#{query.table_name}.#{field}"
        end
      end

      class ArelSelect < Select
        include ArelColumnizer
        OPS = {
          'Eq' => '=', 'In' => 'IN', 'Nin' => 'NOT IN', 'Neq' => '!=',
        }

        def to_query
          args = [lhs, rhs].select { |s| s.is_a? Value }.map(&:value)
          source.to_query.where(
            "#{columnize lhs} #{OPS[kind]} #{columnize rhs}",
            *args)
        end
      end

      class ArelJoin < Join
        include ArelColumnizer
        def to_query
          left.to_query.joins(
            "INNER JOIN #{rcol.source.model.table_name} ON " +
            "#{columnize lcol} = #{columnize rcol}"
          )
        end
      end

      class DataFilter
        CLASSES = {
          select: ArelSelect,
          source: ArelSource,
          join:   ArelJoin,
          union:  ArelUnion,
          imm:    Value,
          field:  Proj,
        }
      end
    end
  end
end
