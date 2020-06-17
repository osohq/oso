# frozen_string_literal: true

require 'osohq/oso/version'
require 'osohq/polar'

module Osohq
  module Oso
    class Oso
      def initialize
        @polar = Osohq::Polar::Polar.new
      end

      def allow(actor:, action:, resource:)
        polar.query_pred('allow', args: [actor, action, resource]).next
        true
      rescue StopIteration
        false
      end

      def load_file(file)
        polar.load_file(file)
      end

      private

      attr_reader :polar
    end
  end
end
