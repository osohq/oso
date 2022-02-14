# frozen_string_literal: true

module Oso
  module Polar
    # Base error type for Oso::Polar.
    class Error < ::Oso::Error
      attr_reader :stack_trace

      # @param message [String]
      # @param details [Hash]
      def initialize(message = nil, details: nil)
        @details = details
        @stack_trace = details&.fetch('stack_trace', nil)
        super(message)
      end
    end

    # Expected to find an FFI error to convert into a Ruby exception but found none.
    class FFIErrorNotFound < Error; end

    # Generic runtime exception.
    class PolarRuntimeError < Error; end

    # Errors from across the FFI boundary.

    class UnsupportedError < PolarRuntimeError; end
    class PolarTypeError < PolarRuntimeError; end
    class StackOverflowError < PolarRuntimeError; end

    # Errors originating from this side of the FFI boundary.

    class UnregisteredClassError < PolarRuntimeError; end
    class UnregisteredInstanceError < PolarRuntimeError; end
    class DuplicateInstanceRegistrationError < PolarRuntimeError; end

    # TODO: I think this should probably have some arguments to say what the call is
    class InvalidCallError < PolarRuntimeError; end
    class InvalidConstructorError < PolarRuntimeError; end
    class InvalidIteratorError < PolarRuntimeError; end
    class InvalidQueryTypeError < PolarRuntimeError; end
    class NullByteInPolarFileError < PolarRuntimeError; end
    class UnexpectedPolarTypeError < PolarRuntimeError; # rubocop:disable Style/Documentation
      def initialize(tag)
        if tag == 'Expression'
          super(UNEXPECTED_EXPRESSION_MESSAGE)
        else
          super(tag)
        end
      end
    end
    class InlineQueryFailedError < PolarRuntimeError; # rubocop:disable Style/Documentation
      # @param source [String]
      def initialize(source)
        super("Inline query failed: #{source}")
      end
    end
    class PolarFileExtensionError < PolarRuntimeError # rubocop:disable Style/Documentation
      def initialize(file)
        super("Polar files must have .polar extension. Offending file: #{file}")
      end
    end
    class PolarFileNotFoundError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param file [String]
      def initialize(file)
        super("Could not find file: #{file}")
      end
    end
    class DuplicateClassAliasError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param name [String]
      # @param old [Class]
      # @param new [Class]
      def initialize(name:, old:, new:)
        super("Attempted to alias #{new} as '#{name}', but #{old} already has that alias.")
      end
    end

    class UnimplementedOperationError < PolarRuntimeError # rubocop:disable Style/Documentation
      def initialize(operation)
        super("#{operation} are unimplemented in the oso Ruby library")
      end
    end

    # Generic operational exception.
    class OperationalError < Error; end
    class UnknownError < OperationalError; end

    # Catch-all for a parsing error that doesn't match any of the more specific types.
    class ParseError < Error
      class ExtraToken < ParseError; end
      class IntegerOverflow < ParseError; end
      class InvalidTokenCharacter < ParseError; end
      class InvalidToken < ParseError; end
      class UnrecognizedEOF < ParseError; end
      class UnrecognizedToken < ParseError; end
    end

    class ValidationError < Error; end

    # @!visibility private
    UNEXPECTED_EXPRESSION_MESSAGE = <<~MSG
      Received Expression from Polar VM. The Expression type is not yet supported in this language.

      This may mean you performed an operation in your policy over an unbound variable.
    MSG
  end
end
