# frozen_string_literal: true

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

      # Register a Ruby method call, wrapping the call result in a generator if
      # it isn't already one.
      #
      # @param method [#to_sym]
      # @param call_id [Integer]
      # @param instance [Hash<String, Object>]
      # @param args [Array<Hash>]
      # @raise [InvalidCallError] if the method doesn't exist on the instance or
      #   the args passed to the method are invalid.
      def register_call(attribute, call_id:, instance:, args:) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        return if calls.key?(call_id)

        instance = host.to_ruby(instance)
        if args.nil?
          result = instance.__send__(attribute)
        else
          args = args.map { |a| host.to_ruby(a) }
          result = instance.__send__(attribute, *args)
        end
        result = [result].to_enum unless result.is_a? Enumerator # Call must be a generator.
        calls[call_id] = result.lazy
      rescue ArgumentError, NoMethodError
        raise InvalidCallError
      end

      # Send next result of Ruby method call across FFI boundary.
      #
      # @param result [String]
      # @param call_id [Integer]
      # @raise [Error] if the FFI call raises one.
      def call_result(result, call_id:)
        ffi_query.call_result(result, call_id: call_id)
      end

      # Retrieve the next result from a registered call and pass it to {#to_polar_term}.
      #
      # @param id [Integer]
      # @return [Hash]
      # @raise [StopIteration] if the call has been exhausted.
      def next_call_result(id)
        host.to_polar_term(calls[id].next)
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
      def handle_call(attribute, call_id:, instance:, args:)
        register_call(attribute, call_id: call_id, instance: instance, args: args)
        result = JSON.dump(next_call_result(call_id))
        call_result(result, call_id: call_id)
      rescue InvalidCallError => e
        application_error(e.message)
        call_result(nil, call_id: call_id)
      rescue StopIteration
        call_result(nil, call_id: call_id)
      end

      def handle_make_external(data) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        id = data['instance_id']
        raise DuplicateInstanceRegistrationError, id if host.instance? id

        constructor = data['constructor']['value']
        raise InvalidConstructorError unless constructor.key? 'Call'

        cls_name = constructor['Call']['name']
        args = constructor['Call']['args'].map { |arg| host.to_ruby(arg) }
        kwargs = constructor['Call']['kwargs']
        kwargs = if kwargs.nil?
                   {}
                 else
                   Hash[kwargs.map { |k, v| [k.to_sym, host.to_ruby(v)] }]
                 end
        host.make_instance(cls_name, args: args, kwargs: kwargs, id: id)
      end

      # Create a generator that can be polled to advance the query loop.
      #
      # @yieldparam [Hash<String, Object>]
      # @return [Enumerator]
      # @raise [Error] if any of the FFI calls raise one.
      def start # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength
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
              args = event.data['args']
              handle_call(attribute, call_id: call_id, instance: instance, args: args)
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
              command = JSON.dump(host.to_polar_term(input))
              ffi_query.debug_command(command)
            else
              raise "Unhandled event: #{JSON.dump(event.inspect)}"
            end
          end
        end.lazy
      end
    end
  end
end
