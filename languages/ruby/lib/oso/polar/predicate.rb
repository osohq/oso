# frozen_string_literal: true

class Oso
  class Polar
    # Polar predicate.
    class Predicate
      attr_reader :name, :args

      # @param name [String]
      # @param args [Array<Object>]
      def initialize(name, args:)
        @name = name
        @args = args
      end

      # @param other [Predicate]
      # @return [Boolean]
      def ==(other)
        name == other.name && args == other.args
      end

      # @see #==
      alias eql? ==
    end
  end
end
