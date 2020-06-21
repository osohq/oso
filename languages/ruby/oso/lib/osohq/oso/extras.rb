# frozen_string_literal: true

# A resource accessed via HTTP.
module Osohq
  module Oso
    class Http
      def initialize(hostname: nil, path: nil, query: nil)
        @hostname = hostname
        @path = path
        @query = query
      end

      private

      attr_reader :hostname, :path, :query
    end

    # Map from a template string with capture groups of the form
    # `{name}` to a dictionary of the form `{name: captured_value}`
    class PathMapper
      def initialize(template:)
        capture_group = /({([^}]+)})/

        template = template.dup
        template.scan(capture_group).each do |outer, inner|
          template = if inner == '*'
                       template.gsub! outer, '.*'
                     else
                       template.gsub! outer, "(?<#{inner}>[^/]+)"
                     end
        end
        @pattern = /\A#{template}\Z/
      end

      def map(string)
        string.match(pattern)&.named_captures || {}
      end

      private

      attr_reader :pattern
    end
  end
end
