# frozen_string_literal: true

require 'oso/oso'
require 'oso/errors'
require 'oso/polar'
require 'oso/version'

# Top-level namespace for Oso authorization library.
module Oso
  def self.new(not_found_error: NotFoundError, forbidden_error: ForbiddenError, read_action: 'read')
    ::Oso::Oso.new(not_found_error: not_found_error, forbidden_error: forbidden_error, read_action: read_action)
  end
end
