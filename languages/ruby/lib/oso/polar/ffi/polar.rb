# frozen_string_literal: true

require 'json'

module Oso
  module Polar
    module FFI
      # Wrapper class for Polar FFI pointer + operations.
      class Polar < ::FFI::AutoPointer
        attr_accessor :enrich_message

        Rust = Module.new do
          extend ::FFI::Library
          ffi_lib FFI::LIB_PATH

          attach_function :new, :polar_new, [], FFI::Polar
          attach_function :load, :polar_load, [FFI::Polar, :string], CResultVoid
          attach_function :clear_rules, :polar_clear_rules, [FFI::Polar], CResultVoid
          attach_function :next_inline_query, :polar_next_inline_query, [FFI::Polar, :uint32], FFI::Query
          attach_function :new_id, :polar_get_external_id, [FFI::Polar], :uint64
          attach_function :new_query_from_str, :polar_new_query, [FFI::Polar, :string, :uint32], CResultQuery
          attach_function :new_query_from_term, :polar_new_query_from_term, [FFI::Polar, :string, :uint32], CResultQuery
          attach_function :register_constant, :polar_register_constant, [FFI::Polar, :string, :string], CResultVoid
          attach_function :register_mro, :polar_register_mro, [FFI::Polar, :string, :string], CResultVoid
          attach_function :next_message, :polar_next_polar_message, [FFI::Polar], CResultString
          attach_function :free, :polar_free, [FFI::Polar], :int32
          attach_function :result_free, :result_free, [:pointer], :int32
          attach_function(
            :build_filter_plan,
            :polar_build_filter_plan,
            [FFI::Polar, :string, :string, :string, :string],
            CResultString
          )
        end
        private_constant :Rust

        # @return [FFI::Polar]
        # @raise [FFI::Error] if the FFI call returns an error.
        def self.create
          Rust.new
        end

        def build_filter_plan(types, partials, variable, class_tag)
          types = JSON.dump(types)
          partials = JSON.dump(partials)
          plan = Rust.build_filter_plan(self, types, partials, variable, class_tag)
          process_messages
          plan = check_result plan
          # TODO(gw) more error checking?
          JSON.parse plan
        end

        # @param sources [Array<Source>]
        # @raise [FFI::Error] if the FFI call returns an error.
        def load(sources)
          loaded = Rust.load(self, JSON.dump(sources))
          process_messages
          check_result loaded
        end

        # @raise [FFI::Error] if the FFI call returns an error.
        def clear_rules
          cleared = Rust.clear_rules(self)
          process_messages
          check_result cleared
        end

        # @return [FFI::Query] if there are remaining inline queries.
        # @return [nil] if there are no remaining inline queries.
        # @raise [FFI::Error] if the FFI call returns an error.
        def next_inline_query
          query = Rust.next_inline_query(self, 0)
          process_messages
          query.null? ? nil : query
        end

        # @return [Integer]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_id
          Rust.new_id(self)
        end

        # @param str [String] Query string.
        # @return [FFI::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_str(str)
          query = Rust.new_query_from_str(self, str, 0)
          process_messages
          check_result query
        end

        # @param term [Hash<String, Object>]
        # @return [FFI::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_term(term)
          query = Rust.new_query_from_term(self, JSON.dump(term), 0)
          process_messages
          check_result query
        end

        # @param name [String]
        # @param value [Hash<String, Object>]
        # @raise [FFI::Error] if the FFI call returns an error.
        def register_constant(value, name:)
          registered = Rust.register_constant(self, name, JSON.dump(value))
          check_result registered
        end

        # @param name [String]
        # @param mro [Array<Integer>]
        # @raise [FFI::Error] if the FFI call returns an error.
        def register_mro(name, mro)
          registered = Rust.register_mro(self, name, JSON.dump(mro))
          check_result registered
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

            message.process(enrich_message)
          end
        end

        def check_result(res)
          result = res[:result]
          error = res[:error]
          Rust.result_free(res)

          raise FFI::Error.get(error, enrich_message) unless error.nil?

          result
        end
      end
    end
  end
end
