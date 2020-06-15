# frozen_string_literal: true

module Osohq
  module Polar
    # Base error type for the Osohq::Polar library.
    class Error < ::RuntimeError; end

    # Expected to find an FFI error to convert into a Ruby exception, but none was found.
    class FFIErrorNotFound < Error; end

    # Generic runtime exception.
    class PolarRuntimeError < Error
      class Serialization < PolarRuntimeError; end
      class Unsupported < PolarRuntimeError; end
      class TypeError < PolarRuntimeError; end
      class StackOverflow < PolarRuntimeError; end
    end

    class OperationalError < Error
      class Unknown < OperationalError; end
    end

    # Catch-all for a parsing error that doesn't match any of the more specific types.
    class ParseError < Error
      # @param [Hash] details about the error
      # @option details [String] :char Character in question.
      # @option details [Array<(Integer, Integer)>] :pos Position of the error.
      # @option details [String] :token Token in question.
      def initialize(**details)
        super(details)
      end

      class ExtraToken < ParseError; end
      class IntegerOverflow < ParseError; end
      class InvalidTokenCharacter < ParseError; end
      class InvalidToken < ParseError; end
      class UnrecognizedEOF < ParseError; end
      class UnrecognizedToken < ParseError; end
    end
  end
end
