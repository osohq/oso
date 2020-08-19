# frozen_string_literal: true

module Oso
  module Polar
    module FFI
      # Wrapper class for Message FFI pointer + operations.
      class Message < ::FFI::AutoPointer
        # @return [String]
        def to_s
          @to_s ||= read_string.force_encoding('UTF-8')
        end

        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :get, :polar_get_message, [], Message
          attach_function :free, :string_free, [Message], :int32
        end

        # rubocop:disable Metrics/MethodLength
        def self.process_messages
          loop do
            message_ptr = Rust.get
            break if message_ptr.null?

            message = JSON.parse(message_ptr.to_s)
            kind = message['kind']
            msg = message['msg']

            case kind
            when 'Print'
              puts(msg)
            when 'Warning'
              puts('[warning] %<msg>s')
            end
          end
        end
        # rubocop:enable Metrics/MethodLength

        private_constant :Rust
      end
    end
  end
end
