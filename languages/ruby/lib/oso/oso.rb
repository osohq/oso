# frozen_string_literal: true

require_relative 'polar/polar'

module Oso
  # oso authorization API.
  class Oso < Polar::Polar
    def initialize
      super
      register_class(Http, name: 'Http')
      register_class(PathMapper, name: 'PathMapper')
    end

    # Query the knowledge base to determine whether an actor is allowed to
    # perform an action upon a resource.
    #
    # @param actor [Object] Subject.
    # @param action [Object] Verb.
    # @param resource [Object] Object.
    # @return [Boolean] An access control decision.
    def allowed?(actor:, action:, resource:)
      query_rule('allow', actor, action, resource).next
      true
    rescue StopIteration
      false
    end
  end
end
