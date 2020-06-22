# frozen_string_literal: true

require 'oso/http'
require 'oso/path_mapper'
require 'oso/polar'
require 'oso/version'

# Oso authorization API.
class Oso
  def initialize
    @polar = Polar.new
    register_class(Http, as: 'Http')
    register_class(PathMapper, as: 'PathMapper')
  end

  def load_file(file)
    polar.load_file(file)
  end

  def load_str(str)
    polar.load_str(str)
  end

  def register_class(cls, as: nil) # rubocop:disable Naming/MethodParameterName
    if block_given?
      polar.register_class(cls, as: as, from_polar: Proc.new)
    else
      polar.register_class(cls, as: as)
    end
  end

  def allow(actor:, action:, resource:)
    polar.query_pred('allow', args: [actor, action, resource]).next
    true
  rescue StopIteration
    false
  end

  private

  attr_reader :polar
end
