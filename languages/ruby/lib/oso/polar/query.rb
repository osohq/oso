# frozen_string_literal: true

require 'json'

module Oso
  module Polar
    # A single Polar query.
    class Query # rubocop:disable Metrics/ClassLength
      # @return [Enumerator]
      attr_reader :results

      # @param ffi_query [FFI::Query]
      # @param host [Oso::Polar::Host]
      def initialize(ffi_query, host:)
        @calls = {}
        @ffi_query = ffi_query
        @host = host
        @results = start
      end

      private

      # @return [Hash<Integer, Enumerator>]
      attr_reader :calls
      # @return [FFI::Query]
      attr_reader :ffi_query
      # @return [Host]
      attr_reader :host

      # Send result of predicate check across FFI boundary.
      #
      # @param result [Boolean]
      # @param call_id [Integer]
      # @raise [Error] if the FFI call raises one.
      def question_result(result, call_id:)
        ffi_query.question_result(result, call_id: call_id)
      end

      # Send next result of Ruby method call across FFI boundary.
      #
      # @param result [String]
      # @param call_id [Integer]
      # @raise [Error] if the FFI call raises one.
      def call_result(result, call_id:)
        ffi_query.call_result(result, call_id: call_id)
      end

      # Retrieve the next result from a registered call and pass it to {#to_polar}.
      #
      # @param id [Integer]
      # @return [Hash]
      # @raise [StopIteration] if the call has been exhausted.
      def next_call_result(id)
        host.to_polar(calls[id].next)
      end

      # Send application error across FFI boundary.
      #
      # @param message [String]
      # @raise [Error] if the FFI call raises one.
      def application_error(message)
        ffi_query.application_error(message)
      end

      # Fetch the next result from calling a Ruby method and prepare it for
      # transmission across the FFI boundary.
      #
      # @param method [#to_sym]
      # @param args [Array<Hash>]
      # @param call_id [Integer]
      # @param instance [Hash<String, Object>]
      # @raise [Error] if the FFI call raises one.
      def handle_call(attribute, call_id:, instance:, args:, kwargs:)
        instance = host.to_ruby(instance)
        args = args.map { |a| host.to_ruby(a) }
        kwargs = Hash[kwargs.map { |k, v| [k.to_sym, host.to_ruby(v)] }]
        # The kwargs.empty? check is for Ruby < 2.7.
        result = if kwargs.empty?
                   instance.__send__(attribute, *args)
                 else
                   instance.__send__(attribute, *args, **kwargs)
                 end
        result = JSON.dump(host.to_polar(result))
        call_result(result, call_id: call_id)
      rescue ArgumentError, NoMethodError => e
        application_error(e.message)
        call_result(nil, call_id: call_id)
      end
      
      def handle_next_external(call_id, term)
        if not calls.key? call_id
          value = host.to_ruby term
          calls[call_id] = value.lazy
        end

        result = JSON.dump(next_call_result(call_id))
        call_result(result, call_id: call_id)      
      rescue StopIteration
        call_result(nil, call_id: call_id)
      end

      def handle_make_external(data) # rubocop:disable Metrics/AbcSize
        id = data['instance_id']
        raise DuplicateInstanceRegistrationError, id if host.instance? id

        constructor = data['constructor']['value']
        raise InvalidConstructorError unless constructor.key? 'Call'

        cls_name = constructor['Call']['name']
        args = constructor['Call']['args'].map { |arg| host.to_ruby(arg) }
        kwargs = constructor['Call']['kwargs'] || {}
        kwargs = Hash[kwargs.map { |k, v| [k.to_sym, host.to_ruby(v)] }]
        host.make_instance(cls_name, args: args, kwargs: kwargs, id: id)
      end

      # Create a generator that can be polled to advance the query loop.
      #
      # @yieldparam [Hash<String, Object>]
      # @return [Enumerator]
      # @raise [Error] if any of the FFI calls raise one.
      def start # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength, Metrics/PerceivedComplexity
        Enumerator.new do |yielder| # rubocop:disable Metrics/BlockLength
          loop do # rubocop:disable Metrics/BlockLength
            event = ffi_query.next_event
            case event.kind
            when 'Done'
              break
            when 'Result'
              yielder << event.data['bindings'].transform_values { |v| host.to_ruby(v) }
            when 'MakeExternal'
              handle_make_external(event.data)
            when 'ExternalCall'
              call_id = event.data['call_id']
              instance = event.data['instance']
              attribute = event.data['attribute']
              args = event.data['args'] || []
              kwargs = event.data['kwargs'] || {}
              handle_call(attribute, call_id: call_id, instance: instance, args: args, kwargs: kwargs)
            when 'ExternalIsSubSpecializer'
              instance_id = event.data['instance_id']
              left_tag = event.data['left_class_tag']
              right_tag = event.data['right_class_tag']
              answer = host.subspecializer?(instance_id, left_tag: left_tag, right_tag: right_tag)
              question_result(answer, call_id: event.data['call_id'])
            when 'ExternalIsa'
              instance = event.data['instance']
              class_tag = event.data['class_tag']
              answer = host.isa?(instance, class_tag: class_tag)
              question_result(answer, call_id: event.data['call_id'])
            when 'ExternalUnify'
              left_instance_id = event.data['left_instance_id']
              right_instance_id = event.data['right_instance_id']
              answer = host.unify?(left_instance_id, right_instance_id)
              question_result(answer, call_id: event.data['call_id'])
            when 'Debug'
              puts event.data['message'] if event.data['message']
              print 'debug> '
              begin
                input = $stdin.readline.chomp.chomp(';')
              rescue EOFError
                next
              end
              command = JSON.dump(host.to_polar(input))
              ffi_query.debug_command(command)
            when 'ExternalOp'
              raise UnimplementedOperationError, 'comparison operators'
            when 'NextExternal'
              call_id = event.data['call_id']
              term = event.data['term']
              handle_next_external(call_id, term)
            else
              raise "Unhandled event: #{JSON.dump(event.inspect)}"
            end
          end
        end.lazy
      end
    end
  end
end
