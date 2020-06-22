# frozen_string_literal: true

require 'oso/http'
require 'oso/oso'
require 'oso/path_mapper'
require 'oso/polar'
require 'oso/version'

# Top-level namespace for oso authorization library.
module Oso
  def self.new
    ::Oso::Oso.new
  end
end
