# frozen_string_literal: true

require_relative 'polar/polar'

module Oso
  # oso authorization API.
  class Oso < Polar::Polar
    def initialize
      super
    end
  end
end
