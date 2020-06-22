# frozen_string_literal: true

require 'ffi'

module Oso
  module Oso
    module Polar
      module FFI
        LIB = ::FFI::Platform::LIBPREFIX + 'polar.' + ::FFI::Platform::LIBSUFFIX
        LIB_PATH = File.expand_path(File.join(__dir__, "../../../../../../target/debug/#{LIB}"))

        # Wrapper classes defined upfront to fix Ruby loading issues. Actual
        # implementations live in the sibling `ffi/` directory and are `require`d
        # at the bottom of this file.

        # Wrapper class for Polar FFI pointer + operations.
        class Polar < ::FFI::AutoPointer
          def self.release(ptr)
            Rust.free(ptr) unless ptr.null?
          end
        end
        # Wrapper class for Query FFI pointer + operations.
        class Query < ::FFI::AutoPointer
          def self.release(ptr)
            Rust.free(ptr) unless ptr.null?
          end
        end
        # Wrapper class for QueryEvent FFI pointer + operations.
        class QueryEvent < ::FFI::AutoPointer
          def self.release(ptr)
            Rust.free(ptr) unless ptr.null?
          end
        end
        # Wrapper class for Error FFI pointer + operations.
        class Error < ::FFI::AutoPointer
          def self.release(ptr)
            Rust.free(ptr)
          end
        end
      end
      private_constant :FFI
    end
  end
end

require 'oso/oso/polar/ffi/polar'
require 'oso/oso/polar/ffi/query'
require 'oso/oso/polar/ffi/query_event'
require 'oso/oso/polar/ffi/error'
