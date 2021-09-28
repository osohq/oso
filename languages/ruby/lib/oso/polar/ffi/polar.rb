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
          attach_function :load, :polar_load, [FFI::Polar, :string], :int32
          attach_function :clear_rules, :polar_clear_rules, [FFI::Polar], :int32
          attach_function :next_inline_query, :polar_next_inline_query, [FFI::Polar, :uint32], FFI::Query
          attach_function :new_id, :polar_get_external_id, [FFI::Polar], :uint64
          attach_function :new_query_from_str, :polar_new_query, [FFI::Polar, :string, :uint32], FFI::Query
          attach_function :new_query_from_term, :polar_new_query_from_term, [FFI::Polar, :string, :uint32], FFI::Query
          attach_function :register_constant, :polar_register_constant, [FFI::Polar, :string, :string], :int32
          attach_function :register_mro, :polar_register_mro, [FFI::Polar, :string, :string], :int32
          attach_function :next_message, :polar_next_polar_message, [FFI::Polar], FFI::Message
          attach_function :free, :polar_free, [FFI::Polar], :int32
          attach_function(
            :build_filter_plan,
            :polar_build_filter_plan,
            [FFI::Polar, :string, :string, :string, :string],
            :string
          )
        end
        private_constant :Rust

        # @return [FFI::Polar]
        # @raise [FFI::Error] if the FFI call returns an error.
        def self.create
          polar = Rust.new
          handle_error if polar.null?

          polar
        end

        def build_filter_plan(types, partials, variable, class_tag)
          types = JSON.dump(types)
          partials = JSON.dump(partials)
          plan = Rust.build_filter_plan(self, types, partials, variable, class_tag)
          process_messages
          handle_error if plan.nil?
          # TODO(gw) more error checking?
          JSON.parse plan
        end

        # @param sources [Array<Source>]
        # @raise [FFI::Error] if the FFI call returns an error.
        def load(sources)
          loaded = Rust.load(self, JSON.dump(sources))
          process_messages
          handle_error if loaded.zero?
        end

        # @raise [FFI::Error] if the FFI call returns an error.
        def clear_rules
          cleared = Rust.clear_rules(self)
          process_messages
          handle_error if cleared.zero?
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
          id = Rust.new_id(self)
          # TODO(gj): I don't think this error check is correct. If getting a new ID fails on the
          # Rust side, it'll probably surface as a panic (e.g., the KB lock is poisoned).
          handle_error if id.zero?

          id
        end

        # @param str [String] Query string.
        # @return [FFI::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_str(str)
          query = Rust.new_query_from_str(self, str, 0)
          process_messages
          handle_error if query.null?

          query
        end

        # @param term [Hash<String, Object>]
        # @return [FFI::Query]
        # @raise [FFI::Error] if the FFI call returns an error.
        def new_query_from_term(term)
          query = Rust.new_query_from_term(self, JSON.dump(term), 0)
          process_messages
          handle_error if query.null?

          query
        end

        # @param name [String]
        # @param value [Hash<String, Object>]
        # @raise [FFI::Error] if the FFI call returns an error.
        def register_constant(value, name:)
          registered = Rust.register_constant(self, name, JSON.dump(value))
          handle_error if registered.zero?
        end

        # @param name [String]
        # @param mro [Array<Integer>]
        # @raise [FFI::Error] if the FFI call returns an error.
        def register_mro(name, mro)
          registered = Rust.register_mro(self, name, JSON.dump(mro))
          handle_error if registered.zero?
        end

        def next_message
          Rust.next_message(self)
        end

        def process_messages
          loop do
            message = next_message
            break if message.null?

            message.process(enrich_message)
          end
        end

        def handle_error
          raise FFI::Error.get(enrich_message)
        end
      end
    end
  end
end
