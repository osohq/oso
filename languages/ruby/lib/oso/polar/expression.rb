# frozen_string_literal: true

module Oso
  module Polar
    # Polar expression.
    class Expression
      attr_accessor :operator, :args

      # @param operator [String]
      # @param args [Array<Object>]
      def initialize(operator, args)
        @operator = operator
        @args = args
      end

      # @param other [Expression]
      # @return [Boolean]
      def ==(other)
        operator == other.operator && args == other.args
      end

      # @see #==
      alias eql? ==
    end
  end
end
