# frozen_string_literal: true

module Oso
  module Polar
    # Data filtering interface for Ruby
    module DataFiltering
      # Represents relationships between resources, eg. one-one or one-many
      class Relation
        attr_reader :kind, :other_type, :my_field, :other_field

        # Describe a Relation from one type to another.
        # @param kind [String] The type of relation, either "one" or "many"
        # @param other_type The name or class object of the related type
        # @param my_field The field on this type that matches +other_type+
        # @param other_field The field on +other_type+ that matches this type
        def initialize(kind:, other_type:, my_field:, other_field:)
          @kind = kind
          @other_type = other_type
          @my_field = my_field
          @other_field = other_field
        end
      end
    end
  end
end
