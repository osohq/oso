# frozen_string_literal: true

module Oso
  module Polar
    # Polar variable.
    class Variable
      attr_reader :name

      # @param name [String]
      def initialize(name)
        @name = name
      end

      # @return [String]
      def to_s
        name
      end
    end
  end
end
