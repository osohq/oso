# frozen_string_literal: true

require 'json'

module Oso
  module Polar
    module FFI
      # Wrapper class for Query FFI pointer + operations.
      class Query < ::FFI::AutoPointer
        attr_accessor :enrich_message

        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :debug_command, :polar_debug_command, [FFI::Query, :string], :int32
          attach_function :call_result, :polar_call_result, [FFI::Query, :uint64, :string], :int32
          attach_function :question_result, :polar_question_result, [FFI::Query, :uint64, :int32], :int32
          attach_function :application_error, :polar_application_error, [FFI::Query, :string], :int32
          attach_function :next_event, :polar_next_query_event, [FFI::Query], FFI::QueryEvent
          attach_function :next_message, :polar_next_query_message, [FFI::Query], FFI::Message
          attach_function :source, :polar_query_source_info, [FFI::Query], FFI::Source
          attach_function :free, :query_free, [FFI::Query], :int32
          attach_function :bind, :polar_bind, [FFI::Query, :string, :string], :int32
        end
        private_constant :Rust

        # @param cmd [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def debug_command(cmd)
          res = Rust.debug_command(self, cmd)
          process_messages
          handle_error if res.zero?
        end

        # @param result [String]
        # @param call_id [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def call_result(result, call_id:)
          res = Rust.call_result(self, call_id, result)
          handle_error if res.zero?
        end

        # @param result [Boolean]
        # @param call_id [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def question_result(result, call_id:)
          result = result ? 1 : 0
          res = Rust.question_result(self, call_id, result)
          handle_error if res.zero?
        end

        # @param message [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def application_error(message)
          res = Rust.application_error(self, message)
          handle_error if res.zero?
        end

        # @return [::Oso::Polar::QueryEvent]
        # @raise [FFI::Error] if the FFI call returns an error.
        def next_event
          event = Rust.next_event(self)
          process_messages
          handle_error if event.null?

          ::Oso::Polar::QueryEvent.new(JSON.parse(event.to_s))
        end

        def bind(name, value)
          res = Rust.bind(self, name, JSON.dump(value))
          handle_error if res.zero?
        end

        def next_message
          Rust.next_message(self)
        end

        def process_messages
          loop do
            message = next_message
            break if message.null?

            message.process(enrich_message)
          end
        end

        # @return [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def source
          res = Rust.source(self)
          handle_error if res.null?

          res.to_s
        end

        def handle_error
          raise FFI::Error.get(enrich_message)
        end
      end
    end
  end
end
