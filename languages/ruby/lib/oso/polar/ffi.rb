# frozen_string_literal: true

require 'ffi'

module Oso
  module Polar
    # FFI classes shared between all ffi/*.rb modules
    module FFI
      LIB =
        case ::FFI::Platform::OS
        when /darwin/
          'libpolar.dylib'
        when /windows|cygwin|msys/
          'polar.dll'
        else
          "libpolar-#{::FFI::Platform::ARCH}.so"
        end

      LIB_PATH = File.expand_path(File.join(__dir__, "../../../ext/oso-oso/lib/#{LIB}"))

      # Wrapper classes defined upfront to fix Ruby loading issues. Actual
      # implementations live in the sibling `ffi/` directory and are `require`d
      # at the bottom of this file.

      # Wrapper class for Polar FFI pointer + operations.
      class Polar < ::FFI::AutoPointer
        def zero?
          null?
        end

        def self.release(ptr)
          Rust.free(ptr) unless ptr.null?
        end
      end
      # Wrapper class for Query FFI pointer + operations.
      class Query < ::FFI::AutoPointer
        def zero?
          null?
        end

        def self.release(ptr)
          Rust.free(ptr) unless ptr.null?
        end
      end

      # Wrapper class for Rust strings FFI pointer + operations.
      class RustString < ::FFI::AutoPointer
        def zero?
          null?
        end

        def self.release(ptr)
          Rust.free(ptr) unless ptr.null?
        end
      end

      # Helper method to generate a Result type for different
      # inner types
      def self.result(result_klass)
        Class.new(::FFI::Struct) do
          layout :result, result_klass, :error, RustString
        end.by_ref
      end

      # Defines the result type version of
      # each of these structs
      # result(T) => { result: T, error: string }
      CResultVoid = result(:int)
      CResultString = result(RustString)
      CResultQuery = result(Query)
    end
    private_constant :FFI
  end
end

require 'oso/polar/ffi/polar'
require 'oso/polar/ffi/query'
require 'oso/polar/ffi/error'
require 'oso/polar/ffi/rust_string'
