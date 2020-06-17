# frozen_string_literal: true

require 'ffi'

module Osohq
  module Polar
    module FFI
      LIB = ::FFI::Platform::LIBPREFIX + 'polar.' + ::FFI::Platform::LIBSUFFIX
      LIB_PATH = File.expand_path(File.join(__dir__, "../../../../../../target/debug/#{LIB}"))
      # Defined upfront to fix Ruby loading issues.
      class Polar < ::FFI::AutoPointer
        def self.release(ptr)
          Rust.free(ptr) unless ptr.null?
        end
      end
      # Defined upfront to fix Ruby loading issues.
      class Query < ::FFI::AutoPointer
        def self.release(ptr)
          Rust.free(ptr) unless ptr.null?
        end
      end
      # Defined upfront to fix Ruby loading issues.
      class QueryEvent < ::FFI::AutoPointer
        def self.release(ptr)
          Rust.free(ptr) unless ptr.null?
        end
      end
      # Defined upfront to fix Ruby loading issues.
      class Error < ::FFI::AutoPointer
        def self.release(ptr)
          Rust.free(ptr)
        end
      end

      # TODO(gj): document
      class Polar < ::FFI::AutoPointer
        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :new, :polar_new, [], Polar
          attach_function :load, :polar_load, [Polar, :string], :int32
          attach_function :next_inline_query, :polar_next_inline_query, [Polar], FFI::Query
          attach_function :new_id, :polar_get_external_id, [Polar], :uint64
          attach_function :new_query_from_str, :polar_new_query, [Polar, :string], FFI::Query
          attach_function :new_query_from_term, :polar_new_query_from_term, [Polar, :string], FFI::Query
          attach_function :new_query_from_repl, :polar_query_from_repl, [Polar], FFI::Query
          attach_function :free, :polar_free, [Polar], :int32
        end
        private_constant :Rust

        # @return [Polar]
        # @raise [FFI::Error] if the FFI call returns an error.
        def self.create
          polar = Rust.new
          raise FFI::Error.get if polar.null?

          polar
        end

        # @param src [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def load(src)
          raise FFI::Error.get if Rust.load(self, src).zero?
        end

        # @return [FFI::Query] if there are remaining inline queries.
        # @return [nil] if there are no remaining inline queries.
        def next_inline_query
          query = Rust.next_inline_query(self)
          query.null? ? nil : query
        end

        # @return [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_id
          id = Rust.new_id(self)
          # TODO(gj): I don't think this error check is correct. If getting a new ID fails on the
          # Rust side, it'll probably surface as a panic (e.g., the KB lock is poisoned).
          raise FFI::Error.get if id.zero?

          id
        end

        # @param str [String] Query string.
        # @return [Osohq::Polar::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_str(str)
          query = Rust.new_query_from_str(self, str)
          raise FFI::Error.get if query.null?

          query
        end

        # @param term [Term]
        # @return [FFI::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_term(term)
          query = Rust.new_query_from_term(self, JSON.dump(term))
          raise FFI::Error.get if query.null?

          query
        end

        # @return [FFI::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_repl
          query = Rust.new_query_from_repl(self)
          raise FFI::Error.get if query.null?

          query
        end
      end

      # TODO(gj): document
      class Query < ::FFI::AutoPointer
        # TODO(gj): document
        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :debug_command, :polar_debug_command, [Query, :string], :int32
          attach_function :call_result, :polar_call_result, [Query, :uint64, :string], :int32
          attach_function :question_result, :polar_question_result, [Query, :uint64, :int32], :int32
          attach_function :next_event, :polar_next_query_event, [Query], FFI::QueryEvent
          attach_function :free, :query_free, [Query], :int32
        end
        private_constant :Rust

        # @param cmd [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def debug_command(cmd)
          res = Rust.debug_command(self, cmd)
          raise FFI::Error.get if res.zero?
        end

        # @param result [String]
        # @param call_id [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def call_result(result, call_id:)
          res = Rust.call_result(self, call_id, result)
          raise FFI::Error.get if res.zero?
        end

        # @param result [Boolean]
        # @param call_id [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def question_result(result, call_id:)
          result = result ? 1 : 0
          res = Rust.question_result(self, call_id, result)
          raise FFI::Error.get if res.zero?
        end

        # @return [Osohq::Polar::QueryEvent]
        # @raise [FFI::Error] if the FFI call returns an error.
        def next_event
          event = Rust.next_event(self)
          raise FFI::Error.get if event.null?

          Osohq::Polar::QueryEvent.new(JSON.parse(event.to_s))
        end
      end

      # TODO(gj): document
      class QueryEvent < ::FFI::AutoPointer
        def to_s
          @to_s ||= read_string.force_encoding('UTF-8')
        end

        # TODO(gj): document
        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :free, :string_free, [QueryEvent], :int32
        end
        private_constant :Rust
      end

      # TODO(gj): document
      class Error < ::FFI::AutoPointer
        def to_s
          @to_s ||= read_string.force_encoding('UTF-8')
        end

        # TODO(gj): document
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
        def self.get
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
        private_class_method def self.parse_error(kind, details:)
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
        private_class_method def self.runtime_error(kind, details:)
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
    private_constant :FFI
  end
end
