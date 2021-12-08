# frozen_string_literal: true

module Oso
  module Polar
    module Data
      # Abstract data adapter
      #
      # An Adapter has to implement two methods.
      class Adapter
        # Make a query object from a filter
        def build_query(_filter)
          raise "build_query not implemented for #{self}"
        end

        # Make a list of objects from a query
        def execute_query(_query)
          raise "execute_query not implemented for #{self}"
        end
      end
    end
  end
end
