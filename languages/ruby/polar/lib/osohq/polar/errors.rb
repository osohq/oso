# frozen_string_literal: true

require 'json'

require 'osohq/polar/ffi'

module Osohq
  module Polar
    module Errors
      class FreeError < ::RuntimeError; end
      class UnhandledEventError < ::RuntimeError; end
      class PolarRuntimeException < ::RuntimeError
        def initialize(msg = '')
          super
        end
      end
      class Unimplemented < ::RuntimeError; end

      class PolarError
        attr_reader :kind, :data, :subkind
        def initialize(json)
          @kind, @data = [*json][0]
          @subkind = [*data][0]
        end
      end

      def self.get_error
        err_s = FFI.polar_get_error
        err = PolarError.new(JSON.parse(err_s))
        err.kind + ' Error: ' + JSON.dump(err.data)
      ensure
        FFI.string_free(err_s)
      end

      def self.check_result(result)
        if result.zero? || result.nil?
            err_s = Errors.get_error
            raise PolarRuntimeException.new err_s
        end
        result
      end
    end
  end
end
