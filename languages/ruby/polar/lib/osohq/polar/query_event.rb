# frozen_string_literal: true

module Osohq
  module Polar
    # A query event received across the FFI boundary.
    class QueryEvent
      # @return [String]
      attr_reader :kind
      # @return [Hash<String, Object>]
      attr_reader :data

      # @overload initialize(event_data)
      #   @param event_data [Hash<String, Object>]
      # @overload initialize(event_data)
      #   @param event_data [String]
      def initialize(event_data)
        event_data = { event_data => nil } if event_data == 'Done'
        @kind, @data = event_data.first
      end
    end
  end
end
