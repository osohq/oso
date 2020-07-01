# frozen_string_literal: true

module Oso
  module Polar
    # A single Polar query.
    class Query
      attr_reader :results

      # @param ffi_instance [FFI::Query]
      # @param polar [Polar]
      def initialize(ffi_instance, polar:)
        @ffi_instance = ffi_instance
        @polar = polar
        @results = start
      end

      private

      # @return [FFI::Query]
      attr_reader :ffi_instance
      # @return [Polar]
      attr_reader :polar

      # Send next result of Ruby method call across FFI boundary.
      #
      # @param result [String]
      # @param call_id [Integer]
      # @raise [Error] if the FFI call raises one.
      def call_result(result, call_id:)
        ffi_instance.call_result(result, call_id: call_id)
      end

      # Send result of predicate check across FFI boundary.
      #
      # @param result [Boolean]
      # @param call_id [Integer]
      # @raise [Error] if the FFI call raises one.
      def question_result(result, call_id:)
        ffi_instance.question_result(result, call_id: call_id)
      end

      # Fetch the next result from calling a Ruby method and prepare it for
      # transmission across the FFI boundary.
      #
      # @param method [#to_sym]
      # @param args [Array<Hash>]
      # @param call_id [Integer]
      # @param instance_id [Integer]
      # @raise [Error] if the FFI call raises one.
      def handle_call(method, args:, call_id:, instance_id:)
        polar.register_call(method, args: args, call_id: call_id, instance_id: instance_id)
        result = JSON.dump(polar.next_call_result(call_id))
        call_result(result, call_id: call_id)
      rescue InvalidCallError, StopIteration
        call_result(nil, call_id: call_id)
        # @TODO: polar line numbers in errors once polar errors are better.
        # raise PolarRuntimeError(f"Error calling {attribute}")
      end

      # Create a generator that can be polled to advance the query loop.
      #
      # @yieldparam [Hash<String, Object>]
      # @return [Enumerator]
      # @raise [Error] if any of the FFI calls raise one.
      def start # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength
        Enumerator.new do |yielder| # rubocop:disable Metrics/BlockLength
          loop do # rubocop:disable Metrics/BlockLength
            event = ffi_instance.next_event
            case event.kind
            when 'Done'
              break
            when 'Result'
              yielder << event.data['bindings'].transform_values { |v| polar.to_ruby(v) }
            when 'MakeExternal'
              id = event.data['instance_id']
              raise DuplicateInstanceRegistrationError, id if polar.instance? id

              cls_name = event.data['instance']['tag']
              fields = event.data['instance']['fields']['fields']
              polar.make_instance(cls_name, fields: fields, id: id)
            when 'ExternalCall'
              call_id = event.data['call_id']
              instance_id = event.data['instance_id']
              method = event.data['attribute']
              args = event.data['args']
              handle_call(method, args: args, call_id: call_id, instance_id: instance_id)
            when 'ExternalIsSubSpecializer'
              instance_id = event.data['instance_id']
              left_tag = event.data['left_class_tag']
              right_tag = event.data['right_class_tag']
              answer = polar.subspecializer?(instance_id, left_tag: left_tag, right_tag: right_tag)
              question_result(answer, call_id: event.data['call_id'])
            when 'ExternalIsa'
              instance_id = event.data['instance_id']
              class_tag = event.data['class_tag']
              answer = polar.isa?(instance_id, class_tag: class_tag)
              question_result(answer, call_id: event.data['call_id'])
            when 'ExternalUnify'
              left_instance_id = event.data['left_instance_id']
              right_instance_id = event.data['right_instance_id']
              answer = polar.unify?(left_instance_id, right_instance_id)
              question_result(answer, call_id: event.data['call_id'])
            when 'Debug'
              puts event.data['message'] if event.data['message']
              print '> '
              input = $stdin.gets.chomp!
              command = JSON.dump(polar.to_polar_term(input))
              ffi_instance.debug_command(command)
            else
              raise "Unhandled event: #{JSON.dump(event.inspect)}"
            end
          end
        end.lazy
      end
    end
  end
end
