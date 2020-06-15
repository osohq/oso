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
      attr_reader :ffi, :instances, :calls

      def initialize
        @ffi = FFI::Polar.create
        @classes = {}
        @class_constructors = {}
        @instances = {}
        @calls = {}
        @load_queue = Set.new
      end

      # Load a Polar string into the KB.
      #
      # @param str [String] Polar string to load.
      def load_str(str:)
        ffi.load_str(str: str)
      end

      def query_str(str:)
        load_queued_files
        query = ffi.new_query_from_str(str: str)
        Query.new(ffi: query, polar: self).results
      end

      def register_class(cls, from_polar = nil)
        cls_name = cls.name
        classes[cls_name] = cls
        class_constructors[cls_name] = from_polar
      end

      def repl
        load_queued_files
        loop do
          query = Query.new(ffi: ffi.new_query_from_repl, polar: self)
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

      def make_external_instance(cls_name, fields, instance_id = nil)
        raise PolarRuntimeError, "Unregistered class: #{cls_name}." unless classes.key?(cls_name)
        raise PolarRuntimeError, "Missing constructor for class: #{cls_name}." unless class_constructors.key?(cls_name)

        cls = classes[cls_name]
        constructor = class_constructors[cls_name]
        begin
          # If constructor is a string, look it up on the class.
          fields = fields.map { |k, v| [k, Term.new(v).to_ruby] }.to_h
          instance = if constructor.nil?
                       cls.new(**fields)
                     else
                       cls.send constructor, **fields
                     end
          cache_instance(instance: instance, id: instance_id)
          instance
        rescue StandardError => e
          raise PolarRuntimeError, "Error constructing instance of #{cls_name}: #{e}"
        end
      end

      def get_instance(id)
        raise PolarRuntimeError, "Unregistered instance: #{id}." unless instances.include?(id)

        instances[id]
      end

      # Clear the KB but retain all registered classes and constructors.
      def clear
        # TODO(gj): Should we clear out instance + call caches as well?
        @ffi = FFI::Polar.create
      end

      # Enqueue a Polar policy file for loading into the KB.
      def load(file)
        unless ['.pol', '.polar'].include? File.extname(file)
          raise PolarRuntimeError, 'Polar files must have .pol or .polar extension.'
        end
        raise PolarRuntimeError, "Could not find file: #{file}" unless File.file?(file)

        load_queue << file
      end

      def to_polar_term(v)
        case v.class
        when TrueClass, FalseClass
          val = { 'Boolean' => v }
        when Integer
          val = { 'Integer' => v }
        when String
          val = { 'String' => v }
        when Array
          val = { 'List' => v.map { |i| to_polar_term(i) } }
        when Hash
          val = {
            'Dictionary' => {
              'fields' => v.map { [k, to_polar_term(v)] }.to_h
            }
          }
          # else
          #   if v.is_a? Predicate
          #     val = {
          #         "Call"=> {
          #             "name"=> v.name,
          #             "args"=> v.args.map { |v| to_polar_term(v)},
          #         }
          #     }
          #   elsif v.is_a? Variable
          #     # This is supported so that we can query for unbound variables
          #     val = {"Symbol"=> v}
          #   else
          #     val = {"ExternalInstance"=> {"instance_id"=> cache_instance(v)}}
          #   end
        end
        { "id": 0, "offset": 0, "value": val }
      end

      private

      #### PRIVATE FIELDS + METHODS ####

      attr_reader :classes, :class_constructors, :load_queue

      def cache_instance(instance:, id: nil)
        id = ffi.new_id if id.nil?
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
      # @param ffi [Osohq::Polar::FFI::Query]
      # @param polar [Osohq::Polar::Polar]
      def initialize(ffi:, polar:)
        @ffi = ffi
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

      def external_call_result(call_id:, result:)
        ffi.call_result(call_id: call_id, result: result, polar: polar.ffi)
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
            external_call_result(call_id: call_id, result: nil)
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
          external_call_result(call_id: call_id, result: stringified)
        rescue StopIteration
          external_call_result(call_id: call_id, result: nil)
        end
      end

      private

      attr_reader :ffi, :polar, :fiber

      def start
        @fiber = Fiber.new do
          loop do
            event = ffi.next_event(polar: polar.ffi)
            case event.kind
            when 'Done'
              break
            when 'Result'
              Fiber.yield event.bindings
            when 'MakeExternal'
              id = event.data['instance_id']
              raise PolarRuntimeError "Instance #{id} already registered." if polar.instances.key?(id)

              cls_name = event.data['instance']['tag']
              fields = event.data['instance']['fields']['fields']
              polar.make_external_instance(cls_name, fields, id)
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

      def initialize(event_data:)
        if event_data == 'Done'
          @kind = 'Done'
        else
          @kind, @data = event_data.first
        end
      end

      def bindings
        # Still skeptical about whether we should have a method that only works for certain types of events
        data['bindings'].sort.map { |k, v| [k, Term.new(v).to_ruby] }.to_h
      end
    end

    # TODO(gj): document
    class Term
      attr_reader :value, :tag, :id, :offset

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

      def initialize(data)
        @id = data['id']
        @offset = data['offset']
        @tag, @value = [*data['value']][0]
      end
    end
  end
end

class TestClass
  def my_method
    1
  end
end

Osohq::Polar::Polar.new.tap do |polar|
  polar.load_str(str: 'f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  puts 'f(x)', polar.query_str(str: 'f(x)').to_a
  puts 'k(x)', polar.query_str(str: 'k(x)').to_a

  polar.load_str(str: 'foo(1, 2); foo(3, 4); foo(5, 6);')
  if polar.query_str(str: 'foo(x, y)').to_a != [{ 'x' => 1, 'y' => 2 }, { 'x' => 3, 'y' => 4 }, { 'x' => 5, 'y' => 6 }]
    raise 'AssertionError'
  end

  t = TestClass.new
  polar.register_class(TestClass)

  polar.load_str(str: 'external(x) := x = TestClass{}.my_method;')
  results = polar.query_str(str: 'external(1)')
  results.next
  # raise 'AssertionError' if polar.query_str('external(1)').to_a != [{ 'x' => 1 }]
end
