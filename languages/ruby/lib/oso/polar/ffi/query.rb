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

          attach_function :debug_command, :polar_debug_command, [FFI::Query, :string], CResultVoid
          attach_function :call_result, :polar_call_result, [FFI::Query, :uint64, :string], CResultVoid
          attach_function :question_result, :polar_question_result, [FFI::Query, :uint64, :int32], CResultVoid
          attach_function :application_error, :polar_application_error, [FFI::Query, :string], CResultVoid
          attach_function :next_event, :polar_next_query_event, [FFI::Query], CResultString
          attach_function :next_message, :polar_next_query_message, [FFI::Query], CResultString
          attach_function :source, :polar_query_source_info, [FFI::Query], CResultString
          attach_function :free, :query_free, [FFI::Query], :int32
          attach_function :result_free, :result_free, [:pointer], :int32
          attach_function :bind, :polar_bind, [FFI::Query, :string, :string], CResultVoid
        end
        private_constant :Rust

        # @param cmd [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def debug_command(cmd)
          res = Rust.debug_command(self, cmd)
          process_messages
          check_result res
        end

        # @param value [Object]
        # @param call_id [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def call_result(value, call_id:)
          res = Rust.call_result(self, call_id, JSON.dump(value))
          check_result res
        end

        # @param result [Boolean]
        # @param call_id [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def question_result(result, call_id:)
          result = result ? 1 : 0
          res = Rust.question_result(self, call_id, result)
          check_result res
        end

        # @param message [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def application_error(message)
          res = Rust.application_error(self, message)
          check_result res
        end

        # @return [::Oso::Polar::QueryEvent]
        # @raise [FFI::Error] if the FFI call returns an error.
        def next_event
          event = Rust.next_event(self)
          process_messages
          event = check_result event

          ::Oso::Polar::QueryEvent.new(JSON.parse(event.to_s))
        end

        def bind(name, value)
          res = Rust.bind(self, name, JSON.dump(value))
          check_result res
        end

        def next_message
          check_result Rust.next_message(self)
        end

        def process_message(message, enrich_message)
          message = JSON.parse(message.to_s)
          kind = message['kind']
          msg = message['msg']
          msg = enrich_message.call(msg)

          case kind
          when 'Print'
            puts(msg)
          when 'Warning'
            warn(format('[warning] %<msg>s', msg: msg))
          end
        end

        def process_messages
          loop do
            message = next_message
            break if message.null?

            process_message(message, enrich_message)
          end
        end

        # @return [String]
        # @raise [FFI::Error] if the FFI call returns an error.
        def source
          res = Rust.source(self)
          res = check_result res

          res.to_s
        end

        # Unwrap the result by (a) extracting the pointers for
        # result and error, (b) freeing the result pointers, and then
        # (c) either returning the result pointer, or constructing and
        # raising the error.
        def check_result(res)
          result = res[:result]
          error = res[:error]
          Rust.result_free(res)

          raise 'internal error: both result and error pointers are not nil' if !error.nil? && !result.nil?
          raise FFI::Error.get(error, enrich_message) unless error.nil?

          result
        end
      end
    end
  end
end
