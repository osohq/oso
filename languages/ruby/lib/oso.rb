# frozen_string_literal: true

require 'oso/oso'
require 'oso/errors'
require 'oso/polar'
require 'oso/version'

# Top-level namespace for Oso authorization library.
module Oso
  def self.new
    ::Oso::Oso.new
  end
end
