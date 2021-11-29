# frozen_string_literal: true

require 'json'

module Oso
  module Polar
    module FFI
      # Wrapper class for Rust strings.
      #
      # Since we force all strings to go through this
      # the `AutoPointer` class will handle
      # actually freeing the string when deleting it
      class RustString < ::FFI::AutoPointer
        # @return [String]
        def to_s
          @to_s ||= read_string.force_encoding('UTF-8')
        end

        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :free, :string_free, [RustString], :int32
        end

        private_constant :Rust
      end
    end
  end
end
