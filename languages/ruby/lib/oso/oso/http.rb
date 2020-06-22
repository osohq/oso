# frozen_string_literal: true

module Oso
  module Oso
    # An HTTP resource.
    class Http
      def initialize(hostname: nil, path: nil, query: nil)
        @hostname = hostname
        @path = path
        @query = query
      end

      private

      attr_reader :hostname, :path, :query
    end
  end
end
