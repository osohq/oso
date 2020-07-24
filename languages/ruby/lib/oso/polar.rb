# frozen_string_literal: true

require 'oso/polar/errors'
require 'oso/polar/ffi'
require 'oso/polar/host'
require 'oso/polar/polar'
require 'oso/polar/predicate'
require 'oso/polar/query'
require 'oso/polar/query_event'
require 'oso/polar/variable'

module Oso
  # Top-level namespace for Polar language library.
  module Polar
    def self.new
      ::Oso::Polar::Polar.new
    end
  end
end
