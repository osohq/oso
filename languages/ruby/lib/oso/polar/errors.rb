# frozen_string_literal: true

module Oso
  module Polar
    # Base error type for Oso::Polar.
    class Error < ::RuntimeError
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

    class SerializationError < PolarRuntimeError; end
    class UnsupportedError < PolarRuntimeError; end
    class PolarTypeError < PolarRuntimeError; end
    class StackOverflowError < PolarRuntimeError; end

    # Errors originating from this side of the FFI boundary.

    class UnregisteredClassError < PolarRuntimeError; end
    class MissingConstructorError < PolarRuntimeError; end
    class UnregisteredInstanceError < PolarRuntimeError; end
    class DuplicateInstanceRegistrationError < PolarRuntimeError; end
    class InvalidCallError < PolarRuntimeError; end
    class InvalidConstructorError < PolarRuntimeError; end
    class InvalidQueryTypeError < PolarRuntimeError; end
    class InlineQueryFailedError < PolarRuntimeError; end
    class NullByteInPolarFileError < PolarRuntimeError; end
    class UnexpectedPolarTypeError < PolarRuntimeError; end
    class PolarFileAlreadyLoadedError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param file [String]
      def initialize(file)
        super("File #{file} has already been loaded.")
      end
    end
    class PolarFileContentsChangedError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param file [String]
      def initialize(file)
        super("A file with the name #{file}, but different contents, has already been loaded.")
      end
    end
    class PolarFileNameChangedError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param file [String]
      # @param existing [String]
      def initialize(file, existing)
        super("A file with the same contents as #{file} named #{existing} has already been loaded.")
      end
    end
    class PolarFileExtensionError < PolarRuntimeError # rubocop:disable Style/Documentation
      def initialize
        super('Polar files must have .pol or .polar extension.')
      end
    end
    class PolarFileNotFoundError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param file [String]
      def initialize(file)
        super("Could not find file: #{file}")
      end
    end
    class DuplicateClassAliasError < PolarRuntimeError # rubocop:disable Style/Documentation
      # @param as [String]
      # @param old [Class]
      # @param new [Class]
      def initialize(name:, old:, new:)
        super("Attempted to alias #{new} as '#{name}', but #{old} already has that alias.")
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

    # Generic Polar API exception.
    class ApiError < Error; end
    class ParameterError < ApiError; end
  end
end
