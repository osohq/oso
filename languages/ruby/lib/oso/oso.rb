# frozen_string_literal: true

require_relative 'polar/polar'

module Oso
  # Oso authorization API.
  class Oso < Polar::Polar
    def initialize
      super
      register_class(Http, name: 'Http')
      register_class(PathMapper, name: 'PathMapper')
    end

    def allowed?(actor:, action:, resource:)
      query_rule('allow', actor, action, resource).next
      true
    rescue StopIteration
      false
    end
  end
end
