# frozen_string_literal: true

require 'json'

module Oso
  module Polar
    # A single Polar query.
    class Query # rubocop:disable Metrics/ClassLength
      include Enumerable

      # @param ffi_query [FFI::Query]
      # @param host [Oso::Polar::Host]
      def initialize(ffi_query, host:, bindings: {})
        @calls = {}
        @ffi_query = ffi_query
        ffi_query.enrich_message = host.method(:enrich_message)
        @host = host
        bindings.each { |k, v| ffi_query.bind k, host.to_polar(v) }
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
      def handle_call(attribute, call_id:, instance:, args:, kwargs:) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        instance = host.to_ruby(instance)
        rel = get_relationship(instance.class, attribute)
        return handle_relationship(call_id, instance, rel) unless rel.nil?

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

      def get_field(cls, tag)
        raise unless cls.fields.key? tag

        ref = cls.fields[tag]
        return host.types[ref] unless ref.is_a? ::Oso::Polar::DataFiltering::Relationship

        case ref.kind
        when 'parent'
          host.types[ref.other_type]
        when 'children'
          host.types[Array]
        end
      end

      def handle_external_isa_with_path(data)
        sup = host.types[data['class_tag']]
        bas = host.types[data['base_tag']]
        path = data['path'].map &host.method(:to_ruby)
        sub = path.reduce(bas) { |cls, tag| get_field(cls, tag) }
#        raise "#{path} #{sub.inspect} #{sup.inspect}"
        answer = sub.klass.get <= sup.klass.get
        question_result(answer, call_id: data['call_id'])
      rescue => e
        application_error e.message
        question_result(nil, call_id: data['call_id'])
      end

      def handle_next_external(call_id, iterable)
        unless calls.key? call_id
          value = host.to_ruby iterable
          raise InvalidIteratorError unless value.is_a? Enumerable

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
      def each # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength, Metrics/PerceivedComplexity
        loop do # rubocop:disable Metrics/BlockLength
          event = ffi_query.next_event
          case event.kind
          when 'Done'
            break
          when 'Result'
            yield event.data['bindings'].transform_values { |v| host.to_ruby(v) }
          when 'MakeExternal'
            handle_make_external(event.data)
          when 'ExternalIsaWithPath'
            handle_external_isa_with_path(event.data)
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
          when 'ExternalIsSubclass'
            call_id = event.data['call_id']
            left = event.data['left_class_tag']
            right = event.data['right_class_tag']
            answer = host.subclass?(left_tag: left, right_tag: right)
            question_result(answer, call_id: call_id)
          when 'ExternalIsa'
            instance = event.data['instance']
            class_tag = event.data['class_tag']
            answer = host.isa?(instance, class_tag: class_tag)
            question_result(answer, call_id: event.data['call_id'])
          when 'Debug'
            msg = event.data['message']
            if msg
              msg = host.enrich_message(msg) if msg
              puts msg
            end
            print 'debug> '
            begin
              input = $stdin.readline.chomp.chomp(';')
            rescue EOFError
              next
            end
            command = JSON.dump(host.to_polar(input))
            ffi_query.debug_command(command)
          when 'ExternalOp'
            op = event.data['operator']
            args = event.data['args'].map(&host.method(:to_ruby))
            answer = host.operator(op, args)
            question_result(answer, call_id: event.data['call_id'])
          when 'NextExternal'
            call_id = event.data['call_id']
            iterable = event.data['iterable']
            handle_next_external(call_id, iterable)
          else
            raise "Unhandled event: #{JSON.dump(event.inspect)}"
          end
        end
      end

      private

      def get_relationship(cls, attr)
        typ = host.types[cls]
        return unless typ
        rel = typ.fields[attr]
        return unless rel.is_a? ::Oso::Polar::DataFiltering::Relationship
        rel 
      end

      def handle_relationship(call_id, instance, rel)
        fetcher = host.types[rel.other_type].fetcher
        constraint = ::Oso::Polar::DataFiltering::Constraint.new(
          kind: 'Eq',
          field: rel.other_field,
          value: instance.send(rel.my_field)
        )
        res = fetcher[[constraint]]

        if rel.kind == 'parent'
          raise unless res.length == 1
          res = res[0]
        end

        res = JSON.dump host.to_polar res
        call_result(res, call_id: call_id)
      end
    end
  end
end
