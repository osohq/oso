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

        def self.process_messages()
          loop do
            message_ptr = Rust.get()
            break if message_ptr.null?
            
            message = JSON.parse(message_ptr.to_s)
            kind = message["kind"]
            msg = message["msg"]
            
            if kind == "Print"
              puts(msg)
            elsif kind == "Warning"
              puts("[warning] %s" % msg)
            end
          end
        end

        private_constant :Rust
      end
    end
  end
end
