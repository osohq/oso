# frozen_string_literal: true

module Oso
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
        # @return [::Oso::Polar::Error] if there's an FFI error.
        # @return [::Oso::Polar::FFIErrorNotFound] if there isn't one.
        def self.get # rubocop:disable Metrics/MethodLength
          error = Rust.get
          return ::Oso::Polar::FFIErrorNotFound if error.null?

          error = JSON.parse(error.to_s)
          msg = error['formatted']
          kind, body = error['kind'].first
          subkind, details = body.first
          case kind
          when 'Parse'
            parse_error(subkind, msg: msg, details: details)
          when 'Runtime'
            runtime_error(subkind, msg: msg, details: details)
          when 'Operational'
            operational_error(subkind, msg: msg, details: details)
          when 'Parameter'
            api_error(subkind, msg: msg, details: details)
          end
        end

        # Map FFI parse errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param msg [String]
        # @param details [Hash<String, Object>]
        # @return [::Oso::Polar::ParseError] the object converted into the expected format.
        private_class_method def self.parse_error(kind, msg:, details:) # rubocop:disable Metrics/MethodLength
          case kind
          when 'ExtraToken'
            ::Oso::Polar::ParseError::ExtraToken.new(msg, details: details)
          when 'IntegerOverflow'
            ::Oso::Polar::ParseError::IntegerOverflow.new(msg, details: details)
          when 'InvalidToken'
            ::Oso::Polar::ParseError::InvalidToken.new(msg, details: details)
          when 'InvalidTokenCharacter'
            ::Oso::Polar::ParseError::InvalidTokenCharacter.new(msg, details: details)
          when 'UnrecognizedEOF'
            ::Oso::Polar::ParseError::UnrecognizedEOF.new(msg, details: details)
          when 'UnrecognizedToken'
            ::Oso::Polar::ParseError::UnrecognizedToken.new(msg, details: details)
          else
            ::Oso::Polar::ParseError.new(msg, details: details)
          end
        end

        # Map FFI runtime errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param msg [String]
        # @param details [Hash<String, Object>]
        # @return [::Oso::Polar::PolarRuntimeError] the object converted into the expected format.
        private_class_method def self.runtime_error(kind, msg:, details:) # rubocop:disable Metrics/MethodLength
          case kind
          when 'Serialization'
            ::Oso::Polar::SerializationError.new(msg, details: details)
          when 'Unsupported'
            ::Oso::Polar::UnsupportedError.new(msg, details: details)
          when 'TypeError'
            ::Oso::Polar::PolarTypeError.new(msg, details: details)
          when 'StackOverflow'
            ::Oso::Polar::StackOverflowError.new(msg, details: details)
          when 'FileLoading'
            ::Oso::Polar::FileLoadingError.new(msg, details: details)
          else
            ::Oso::Polar::PolarRuntimeError.new(msg, details: details)
          end
        end

        # Map FFI operational errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param msg [String]
        # @param details [Hash<String, Object>]
        # @return [::Oso::Polar::OperationalError] the object converted into the expected format.
        private_class_method def self.operational_error(kind, msg:, details:)
          case kind
          when 'Unknown' # Rust panics.
            ::Oso::Polar::UnknownError.new(msg, details: details)
          else
            ::Oso::Polar::OperationalError.new(msg, details: details)
          end
        end

        # Map FFI API errors into Ruby exceptions.
        #
        # @param kind [String]
        # @param msg [String]
        # @param details [Hash<String, Object>]
        # @return [::Oso::Polar::ApiError] the object converted into the expected format.
        private_class_method def self.api_error(kind, msg:, details:)
          case kind
          when 'Parameter'
            ::Oso::Polar::ParameterError.new(msg, details: details)
          else
            ::Oso::Polar::ApiError.new(msg, details: details)
          end
        end
      end
    end
  end
end
