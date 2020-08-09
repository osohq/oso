# frozen_string_literal: true

module Oso
  # An HTTP resource.
  class Http
    def initialize(hostname, path, query)
      @hostname = hostname
      @path = path
      @query = query
    end

    private

    attr_reader :hostname, :path, :query
  end
end
