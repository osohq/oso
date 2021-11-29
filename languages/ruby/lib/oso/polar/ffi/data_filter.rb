# frozen_string_literal: true

module Oso
  module Polar
    module FFI
      # Wrapper class for QueryEvent FFI pointer + operations.
      class DataFilter < ::FFI::AutoPointer
        # @return [String]
        def to_s
          @to_s ||= read_string.force_encoding('UTF-8')
        end

        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :free, :string_free, [FFI::DataFilter], :int32
        end
        private_constant :Rust
      end
    end
  end
end
