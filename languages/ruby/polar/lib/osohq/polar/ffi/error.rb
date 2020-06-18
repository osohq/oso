# frozen_string_literal: true

module Osohq
  module Polar
    module FFI
      # Wrapper class for Error FFI pointer + operations.
      class Error < ::FFI::AutoPointer
        def to_s
          @to_s ||= read_string.force_encoding('UTF-8')
        end

        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :get, :polar_get_error, [], Error
          attach_function :free, :string_free, [Error], :int32
        end
        private_constant :Rust

        # Check for an FFI error and convert it into a Ruby exception.
        #
        # @return [Osohq::Polar::Error] if there's an FFI error.
        # @return [Osohq::Polar::FFIErrorNotFound] if there isn't one.
        def self.get # rubocop:disable Metrics/MethodLength
          error = Rust.get
          return Osohq::Polar::FFIErrorNotFound if error.null?

          kind, body = JSON.parse(error.to_s).first
          subkind, details = body.first
          case kind
          when 'Parse'
            parse_error(subkind, details: details)
          when 'Runtime'
            runtime_error(subkind, details: details)
          when 'Operational'
            operational_error(subkind, details: details)
          end
        end

        # Map FFI parse errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param details [Hash<String, Object>]
        # @return [Osohq::Polar::ParseError] the object converted into the expected format.
        private_class_method def self.parse_error(kind, details:) # rubocop:disable Metrics/CyclomaticComplexity, Metrics/MethodLength
          case kind
          when 'ExtraToken'
            Osohq::Polar::ParseError::ExtraToken.new(details)
          when 'IntegerOverflow'
            Osohq::Polar::ParseError::IntegerOverflow.new(details)
          when 'InvalidToken'
            Osohq::Polar::ParseError::InvalidToken.new(details)
          when 'InvalidTokenCharacter'
            Osohq::Polar::ParseError::InvalidTokenCharacter.new(details)
          when 'UnrecognizedEOF'
            Osohq::Polar::ParseError::UnrecognizedEOF.new(details)
          when 'UnrecognizedToken'
            Osohq::Polar::ParseError::UnrecognizedToken.new(details)
          else
            Osohq::Polar::ParseError.new(details)
          end
        end

        # Map FFI runtime errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param details [Hash<String, Object>]
        # @return [Osohq::Polar::PolarRuntimeError] the object converted into the expected format.
        private_class_method def self.runtime_error(kind, details:) # rubocop:disable Metrics/MethodLength
          msg = details['msg']
          case kind
          when 'Serialization'
            Osohq::Polar::SerializationError.new(msg)
          when 'Unsupported'
            Osohq::Polar::UnsupportedError.new(msg)
          when 'TypeError'
            Osohq::Polar::PolarTypeError.new(msg)
          when 'StackOverflow'
            Osohq::Polar::StackOverflowError.new(msg)
          else
            Osohq::Polar::PolarRuntimeError.new(msg)
          end
        end

        # Map FFI operational errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param details [Hash<String, Object>]
        # @return [Osohq::Polar::OperationalError] the object converted into the expected format.
        private_class_method def self.operational_error(kind, details:)
          msg = details['msg']
          case kind
          when 'Unknown' # Rust panics.
            Osohq::Polar::UnknownError.new(msg)
          else
            Osohq::Polar::OperationalError.new(msg)
          end
        end
      end
    end
  end
end
