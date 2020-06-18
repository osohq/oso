# frozen_string_literal: true

module Osohq
  module Polar
    class Query
      # @param ffi_instance [Osohq::Polar::FFI::Query]
      # @param polar [Osohq::Polar::Polar]
      def initialize(ffi_instance, polar:)
        @ffi_instance = ffi_instance
        @polar = polar
        start
      end

      def results
        Enumerator.new do |yielder|
          loop do
            result = fiber.resume
            break if result.nil?

            yielder << result
          end
        end
      end

      private

      attr_reader :ffi_instance, :fiber, :polar

      def call_result(result, call_id:)
        ffi_instance.call_result(result, call_id: call_id)
      end

      def question_result(result, call_id:)
        ffi_instance.question_result(result, call_id: call_id)
      end

      # @param method [#to_sym]
      # @param args [Array<Hash>]
      # @param call_id [Integer]
      # @param instance_id [Integer]
      def handle_call(method, args:, call_id:, instance_id:)
        polar.register_call(method, args: args, call_id: call_id, instance_id: instance_id)
        begin # Return the next result of the call.
          result = JSON.dump(polar.next_call_result(call_id))
          call_result(result, call_id: call_id)
        rescue StopIteration
          call_result(nil, call_id: call_id)
        end
      rescue InvalidCallError
        call_result(nil, call_id: call_id)
        # @TODO: polar line numbers in errors once polar errors are better.
        # raise PolarRuntimeError(f"Error calling {attribute}")
      end

      def start
        @fiber = Fiber.new do
          loop do
            event = ffi_instance.next_event
            case event.kind
            when 'Done'
              break
            when 'Result'
              Fiber.yield(event.data['bindings'].transform_values { |v| polar.to_ruby(v) })
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
            when 'Debug'
              puts event.data['message'] if event.data['message']
              print '> '
              input = gets.chomp!
              command = JSON.dump(polar.to_polar_term(input))
              ffi_instance.debug_command(command)
            else
              raise "Unhandled event: #{JSON.dump(event.inspect)}"
            end
          end
        end
      end
    end
  end
end
