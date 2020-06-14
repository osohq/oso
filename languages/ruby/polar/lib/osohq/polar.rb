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
      attr_reader :pointer, :instances, :calls

      def initialize
        @pointer = FFI.polar_new
        @classes = {}
        @class_constructors = {}
        @instances = {}
        @calls = {}
        @load_queue = Set.new
      end

      # Load a Polar string into the KB.
      #
      # @param str [String] Polar string to load.
      # @return [Boolean] Success / failure.
      def load_str(str)
        FFI.polar_load_str(pointer, str).zero?
      end

      def query_str(str)
        load_queued_files
        Query.from_str(str, self).results
      end

      def register_class(cls, from_polar = nil)
        cls_name = cls.name
        classes[cls_name] = cls
        class_constructors[cls_name] = from_polar
      end

      def repl
        load_queued_files
        loop do
          query = Query.from_repl(self)
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
        raise PolarRuntimeException, "Unregistered class: #{cls_name}." unless classes.key?(cls_name)
        unless class_constructors.key?(cls_name)
          raise PolarRuntimeException, "Missing constructor for class: #{cls_name}."
        end

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
          cache_instance(instance, instance_id)
          instance
        rescue StandardError => e
          raise PolarRuntimeException, "Error constructing instance of #{cls_name}: #{e}"
        end
      end

      def get_instance(id)
        raise PolarRuntimeException, "Unregistered instance: #{id}." unless instances.include?(id)

        instances[id]
      end

      def external_call_result(query, call_id, value)
        result = FFI.polar_external_call_result(pointer, query, call_id, value)
        if result == 0 or result.nil?
           FFI.error
        end
        result
      end

      def external_call(query, data)
        call_id = data['call_id']

        unless calls.key?(call_id)
          instance_id = data['instance_id']
          attribute = data['attribute']
          args = data['args'].map { |arg| Term.new(arg).to_ruby }

          # Lookup the attribute on the instance.
          instance = get_instance(instance_id)
          begin
            if args.empty?
              attribute = instance.send attribute
            else
              attribute = instance.send attribute *args
            end
          rescue StandardError
            external_call_result(polar, query, call_id, nil)
            # @TODO: polar line numbers in errors once polar errors are better.
            # raise PolarRuntimeException(f"Error calling {attribute}")
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
          calls[call_id] = call
        end

        # Return the next result of the call.
        begin
          value = calls[call_id].next
          stringified = JSON.dump(to_polar_term(value))
          external_call_result(query, call_id, stringified)
        rescue StopIteration
          external_call_result(query, call_id, nil)
        end
      end

      # Clear the KB but retain all registered classes and constructors.
      def clear
        # TODO(gj): Should we clear out instance + call caches as well?
        free
        @pointer = FFI.polar_new
      end

      # Enqueue a Polar policy file for loading into the KB.
      def load(file)
        unless ['.pol', '.polar'].include? File.extname(file)
          raise PolarRuntimeException, 'Polar files must have .pol or .polar extension.'
        end
        raise PolarRuntimeException, "Could not find file: #{file}" unless File.file?(file)

        load_queue << file
      end

      def to_polar_term(v)
        case v.class
        when TrueClass, FalseClass
          val = {"Boolean"=> v}
        when Integer
          val = {"Integer"=> v}
        when String
          val = {"String"=> v}
        when Array
          val = {"List"=> v.map{|i| to_polar_term(i)}}
        when Hash
          val = {
              "Dictionary"=> {
                  "fields"=> v.map {[k, to_polar_term(v)]}.to_h
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
        {"id": 0, "offset": 0, "value": val}
      end


      private
      #### PRIVATE FIELDS + METHODS ####

      attr_reader :classes, :class_constructors, :load_queue

      def cache_instance(instance, id = nil)
        if id.nil?
          id = FFI.polar_get_external_id(pointer)
          raise FFIError if id.zero?
        end
        instances[id] = instance
        id
      end

      def free
        raise FreeError if FFI.polar_free(pointer).zero?
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
      def self.from_str(query_str, polar)
        ptr = FFI.polar_new_query(polar.pointer, query_str)
        raise FFI.error if ptr.null?

        new(ptr: ptr, polar: polar)
      end

      def self.from_repl(polar)
        ptr = FFI.polar_query_from_repl(polar.pointer)
        raise FFI.error if ptr.null?

        new(ptr: ptr, polar: polar)
      end

      def initialize(ptr:, polar:)
        @ptr = ptr
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

      attr_reader :ptr, :polar, :fiber

      def free
        raise FreeError if FFI.query_free(ptr).zero?
      end

      def start
        @fiber = Fiber.new do
          begin
            loop do
              string = FFI.polar_query(polar.pointer, ptr)
              raise PolarRuntimeException, Errors.get_error if string.nil?

              event = JSON.parse(string)
              break if event == 'Done'

              event = Event.new(event)
              case event.kind
              when 'Result'
                Fiber.yield event.bindings
              when 'MakeExternal'
                id = event.data['instance_id']
                raise PolarRuntimeException "Instance #{id} already registered." if polar.instances.key?(id)

                cls_name = event.data['instance']['tag']
                fields = event.data['instance']['fields']['fields']
                polar.make_external_instance(cls_name, fields, id)
              when 'ExternalCall'
                polar.external_call(ptr, event.data)
              else
                p event
                raise UnhandledEventError, event.kind
              end
            end
          ensure
            free
          end
        end
      end
    end

    # TODO(gj): document
    class Event
      attr_reader :kind, :data

      def initialize(json)
        @kind, @data = json.first
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
          raise UnimplementedError
        when 'Symbol'
          raise PolarRuntimeException
        else
          raise UnimplementedError
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
  polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  p 'f(x)', polar.query_str('f(x)').to_a
  p 'k(x)', polar.query_str('k(x)').to_a

  polar.load_str('foo(1, 2); foo(3, 4); foo(5, 6);')
  if polar.query_str('foo(x, y)').to_a != [{ 'x' => 1, 'y' => 2 }, { 'x' => 3, 'y' => 4 }, { 'x' => 5, 'y' => 6 }]
    raise 'AssertionError'
  end

  t = TestClass.new
  polar.register_class(TestClass)

  polar.load_str('external(x) := x = TestClass{}.my_method;')
  results = polar.query_str('external(1)')
  results.next
  # raise 'AssertionError' if polar.query_str('external(1)').to_a != [{ 'x' => 1 }]
end
