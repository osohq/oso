# frozen_string_literal: true

require 'json'
require 'set'

require 'osohq/polar/version'
require 'osohq/polar/ffi'
require 'osohq/polar/errors'

module Osohq
  module Polar
    POLAR_TYPES = [Integer, Float, TrueClass, FalseClass, String, Hash, NilClass, Array].freeze

    # TODO(gj): document
    class Polar
      attr_reader :ffi_instance, :instances, :calls

      def initialize
        @ffi_instance = FFI::Polar.create
        @classes = {}
        @constructors = {}
        @instances = {}
        @calls = {}
        @load_queue = Set.new
      end

      # Load a Polar string into the KB.
      #
      # @param str [String] Polar string to load.
      def load_str(str)
        ffi_instance.load_str(str)
      end

      def query_str(str)
        load_queued_files
        query_ffi_instance = ffi_instance.new_query_from_str(str)
        Query.new(query_ffi_instance, polar: self).results
      end

      # @param cls [Class]
      # @param from_polar [Symbol]
      def register_class(cls, from_polar: :new)
        classes[cls.name] = cls
        constructors[cls.name] = from_polar
      end

      def repl
        load_queued_files
        loop do
          query = Query.new(ffi_instance.new_query_from_repl, polar: self)
          results = query.results.to_a
          if results.empty?
            puts 'False'
          else
            results.each do |result|
              puts result
            end
          end
        end
      end

      # @param cls_name [String]
      # @param fields [Hash<String, Object>]
      # @param id [Integer]
      def make_external_instance(cls_name, fields:, id: nil)
        raise UnregisteredClassError, cls_name unless classes.key?(cls_name)
        raise MissingConstructorError, cls_name unless constructors.key?(cls_name)

        fields = fields.transform_values { |v| Term.new(v).to_ruby }
        instance = classes[cls_name].send(constructors[cls_name], **fields)
        cache_instance(instance, id: id)
      rescue StandardError => e
        raise PolarRuntimeError, "Error constructing instance of #{cls_name}: #{e}"
      end

      # @param id [Integer]
      # @return [
      # @raise [UnregisteredInstanceError] if the ID has not been registered.
      def get_instance(id)
        raise UnregisteredInstanceError, id unless instances.key? id

        instances[id]
      end

      # Clear the KB but retain all registered classes and constructors.
      def clear
        # TODO(gj): Should we clear out instance + call caches as well?
        @ffi_instance = FFI::Polar.create
      end

      # Enqueue a Polar policy file for loading into the KB.
      # @param file [String]
      def load(file)
        unless ['.pol', '.polar'].include? File.extname(file)
          raise PolarRuntimeError, 'Polar files must have .pol or .polar extension.'
        end
        raise PolarRuntimeError, "Could not find file: #{file}" unless File.file?(file)

        load_queue << file
      end

      # Turn a Ruby value into a Polar term.
      # @param x [Object]
      def to_polar_term(x)
        case x
        when TrueClass, FalseClass
          val = { 'Boolean' => x }
        when Integer
          val = { 'Integer' => x }
        when String
          val = { 'String' => x }
        when Array
          val = { 'List' => x.map { |el| to_polar_term(el) } }
        when Hash
          val = { 'Dictionary' => { 'fields' => x.transform_values { |v| to_polar_term(v) } } }
        when Predicate
          val = { 'Call' => { 'name' => x.name, 'args' => x.args.map { |el| to_polar_term(el) } } }
        when Variable
          # This is supported so that we can query for unbound variables
          val = { 'Symbol' => x }
        else
          val = { 'ExternalInstance' => { 'instance_id' => cache_instance(x) } }
        end
        { "id": 0, "offset": 0, "value": val }
      end

      private

      #### PRIVATE FIELDS + METHODS ####

      attr_reader :classes, :constructors, :load_queue

      # @param instance [Object]
      # @param id [Integer]
      # @return [Integer]
      def cache_instance(instance, id: nil)
        id = ffi_instance.new_id if id.nil?
        instances[id] = instance
        id
      end

      def load_queued_files
        instances.clear
        load_queue.reject! do |file|
          File.open(file) { |f| load_str(f.read) }
          true
        end
      end
    end

    # TODO(gj): document
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

      def call_result(result, call_id:)
        ffi_instance.call_result(result, call_id: call_id, polar: polar.ffi_instance)
      end

      def external_call(data)
        call_id = data['call_id']

        unless polar.calls.key?(call_id)
          instance_id = data['instance_id']
          attribute = data['attribute']
          args = data['args'].map { |arg| Term.new(arg).to_ruby }

          # Lookup the attribute on the instance.
          instance = polar.get_instance(instance_id)
          begin
            attribute = if args.empty?
                          instance.send attribute
                        else
                          instance.send attribute * args
                        end
          rescue StandardError
            call_result(nil, call_id: call_id)
            # @TODO: polar line numbers in errors once polar errors are better.
            # raise PolarRuntimeError(f"Error calling {attribute}")
            return
          end

          # We now have either a generator or a result.
          # Call must be a generator so we turn anything else into one.
          call = if POLAR_TYPES.include?(attribute.class) || !attribute.is_a?(Enumerable)
                   Enumerator.new do |y|
                     y.yield attribute
                   end
                 elsif attribute.nil?
                   Enumerator.new do |y|
                   end
                 else
                   attribute.each
                 end
          polar.calls[call_id] = call
        end

        # Return the next result of the call.
        begin
          value = polar.calls[call_id].next
          stringified = JSON.dump(polar.to_polar_term(value))
          call_result(stringified, call_id: call_id)
        rescue StopIteration
          call_result(nil, call_id: call_id)
        end
      end

      private

      attr_reader :ffi_instance, :polar, :fiber

      def start
        @fiber = Fiber.new do
          loop do
            event = ffi_instance.next_event(polar.ffi_instance)
            case event.kind
            when 'Done'
              break
            when 'Result'
              Fiber.yield event.bindings
            when 'MakeExternal'
              id = event.data['instance_id']
              raise DuplicateInstanceRegistrationError, id if polar.instances.key?(id)

              cls_name = event.data['instance']['tag']
              fields = event.data['instance']['fields']['fields']
              polar.make_external_instance(cls_name, fields: fields, id: id)
            when 'ExternalCall'
              external_call(event.data)
            else
              p event
              raise "Unhandled event: #{event.kind}"
            end
          end
        end
      end
    end

    # TODO(gj): document
    class QueryEvent
      attr_reader :kind, :data

      def initialize(event_data)
        event_data = { event_data => nil } if event_data == 'Done'
        @kind, @data = event_data.first
      end

      def bindings
        # Still skeptical about whether we should have a method that only works for certain types of events
        data['bindings'].sort.map { |k, v| [k, Term.new(v).to_ruby] }.to_h
      end
    end

    # Polar term.
    class Term
      attr_reader :value, :tag, :id, :offset

      # @param data [Hash<String, Object>]
      # @option data [Integer] :id
      # @option data [Integer] :offset Character offset of the term in its source string.
      # @option data [Hash<String, Object>] :value
      def initialize(data)
        @id = data['id']
        @offset = data['offset']
        @tag, @value = data['value'].first
      end

      def to_ruby
        case tag
        when 'Integer', 'String', 'Boolean'
          value
        when 'List'
          value.map { |term| Term.new(term).to_ruby }
        when 'Dictionary'
          value['fields'].map { |k, v| [k, Term.new(v).to_ruby] }.to_h
        when 'ExternalInstance', 'InstanceLiteral', 'Call'
          raise 'Unimplemented!'
        when 'Symbol'
          raise PolarRuntimeError
        else
          raise 'Unimplemented!'
        end
      end
    end
  end
end

Osohq::Polar::Polar.new.tap do |polar|
  polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  puts 'f(x)', polar.query_str('f(x)').to_a
  puts 'k(x)', polar.query_str('k(x)').to_a

  polar.load_str('foo(1, 2); foo(3, 4); foo(5, 6);')
  if polar.query_str('foo(x, y)').to_a != [{ 'x' => 1, 'y' => 2 }, { 'x' => 3, 'y' => 4 }, { 'x' => 5, 'y' => 6 }]
    raise 'AssertionError'
  end

  class TestClass
    def my_method
      1
    end
  end

  polar.register_class(TestClass)

  polar.load_str('external(x, 3) := x = TestClass{}.my_method;')
  results = polar.query_str('external(1, x)')
  p results.next
  # raise 'AssertionError' if polar.query_str('external(1)').to_a != [{ 'x' => 1 }]
end
