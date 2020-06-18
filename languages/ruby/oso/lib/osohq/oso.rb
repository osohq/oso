# frozen_string_literal: true

require 'osohq/oso/version'
require 'osohq/oso/extras'
require 'osohq/polar'

module Osohq
  module Oso
    class Oso
      def initialize
        @polar = Osohq::Polar::Polar.new
        polar.register_class(Http)
        polar.register_class(PathMapper)
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

      def register_class(cls, &from_polar)
        polar.register_class(cls) { from_polar }
      end

      def load_str(str)
        polar.load_str(str)
      end

      private

      attr_reader :polar
    end
  end
end
