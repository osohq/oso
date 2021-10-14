# frozen_string_literal: true

module Oso
  module Polar
    # Polar pattern.
    class Pattern
      attr_reader :tag, :fields

      # @param tag [String]
      # @param fields [Hash<String, Object>]
      def initialize(tag, fields)
        @tag = tag
        @fields = fields
      end

      # @param other [Pattern]
      # @return [Boolean]
      def ==(other)
        tag == other.tag && fields == other.fields
      end

      # @see #==
      alias eql? ==
    end
  end
end
