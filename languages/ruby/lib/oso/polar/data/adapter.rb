# frozen_string_literal: true

module Oso
  module Polar
    module Data
      # Abstract data adapter
      class Adapter
        def build_query(_filter)
          raise "build_query not implemented for #{self}"
        end

        def exec_query(_query)
          raise "exec_query not implemented for #{self}"
        end
      end
    end
  end
end
