# frozen_string_literal: true

module Osohq
  module Polar
    class QueryEvent
      attr_reader :kind, :data

      def initialize(event_data)
        event_data = { event_data => nil } if event_data == 'Done'
        @kind, @data = event_data.first
      end
    end
  end
end
