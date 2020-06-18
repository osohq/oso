# frozen_string_literal: true

require 'osohq/oso/version'
require 'osohq/oso/extras'
require 'osohq/polar'

module Osohq
  module Oso
    class Oso
      def initialize
        @polar = Osohq::Polar::Polar.new
        register_class(Http)
        register_class(PathMapper)
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
        if from_polar.nil?
          polar.register_class(cls)
        else
          polar.register_class(cls, from_polar)
        end
      end

      def load_str(str)
        polar.load_str(str)
      end

      private

      def query_pred(name, args:)
        polar.query_pred(name, args: args)
      end

      attr_reader :polar
    end
  end
end
