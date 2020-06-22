# frozen_string_literal: true

module Oso
  module Oso
    module Polar
      module FFI
        # Wrapper class for Query FFI pointer + operations.
        class Query < ::FFI::AutoPointer
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

          # @return [::Oso::Oso::Polar::QueryEvent]
          # @raise [FFI::Error] if the FFI call returns an error.
          def next_event
            event = Rust.next_event(self)
            raise FFI::Error.get if event.null?

            ::Oso::Oso::Polar::QueryEvent.new(JSON.parse(event.to_s))
          end
        end
      end
    end
  end
end
