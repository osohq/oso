# frozen_string_literal: true

module Oso
  # Oso authorization API.
  class Oso
    def initialize
      @polar = ::Oso::Polar.new
      register_class(Http, name: 'Http')
      register_class(PathMapper, name: 'PathMapper')
    end

    def load_file(file)
      polar.load_file(file)
    end

    def load_str(str)
      polar.load_str(str)
    end

    def register_class(cls, name: nil) # rubocop:disable Naming/MethodParameterName
      if block_given?
        polar.register_class(cls, name: name, from_polar: Proc.new)
      else
        polar.register_class(cls, name: name)
      end
    end

    def allow(actor:, action:, resource:)
      polar.query_pred('allow', args: [actor, action, resource]).next
      true
    rescue StopIteration
      false
    end

    def query_predicate(name, *args)
      polar.query_pred(name, args: args)
    end

    def load_queued_files
      polar.load_queued_files
    end

    private

    attr_reader :polar
  end
end
