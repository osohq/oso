# frozen_string_literal: true

# A resource accessed via HTTP.
module Osohq
  module Oso
    class Http
      def initialize(path: '', query: {}, hostname: '')
        @path = path
        @query = query
        @hostname = hostname
      end

      def to_str
        host_str = hostname != '' ? "hostname=#{hostname}" : nil
        path_str = path != '' ? "path=#{path}" : nil
        query_str = query != {} ? "query=#{query}" : nil
        field_str = [host_str, path_str, query_str].filter_map { |s| s unless s.nil? }.join(',')
      end
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
        match = string.match(pattern)
        match&.names&.zip(match.captures).to_h
      end

      private

      attr_reader :pattern
    end
  end
end
