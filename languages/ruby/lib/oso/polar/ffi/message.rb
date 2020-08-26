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

          attach_function :free, :string_free, [Message], :int32
        end

        def process
          message = JSON.parse(to_s)
          kind = message['kind']
          msg = message['msg']

          case kind
          when 'Print'
            puts(msg)
          when 'Warning'
            warn(format('[warning] %<msg>s', msg: msg))
          end
        end

        private_constant :Rust
      end
    end
  end
end
