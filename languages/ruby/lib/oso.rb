# frozen_string_literal: true

require 'oso/oso'
require 'oso/polar'
require 'oso/errors'
require 'oso/policy'
require 'oso/version'

# Top-level namespace for oso authorization library.
module Oso
  def self.new
    # TODO: deprecate this method. Instead, users should use:
    # > oso = Oso::Oso.new
    ::Oso::Oso.new
  end
end
