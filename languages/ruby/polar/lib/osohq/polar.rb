# frozen_string_literal: true

require 'json'

require 'osohq/polar/version'
require 'osohq/polar/ffi'
require 'osohq/polar/errors'

module Osohq
  module Polar
    class Polar
      attr_reader :pointer

      def initialize
        @pointer = FFI.polar_new
        @classes = {}
        @class_constructors = {}
        @instances = {}
        @calls = {}
      end

      def load_str(str)
        Errors.check_result(FFI.polar_load_str(pointer, str))
      end

      def query_str(str)
        Query.from_str(str, self).results
      end

      def register_class(cls, from_polar = nil)
        cls_name = cls.name
        classes[cls_name] = cls
        class_constructors[cls_name] = from_polar
      end

      def repl
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

      def make_external_instance(cls_name, fields, instance_id=nil)
        if !classes.key?(cls_name)
            raise PolarRuntimeException.new("Unregistered class: #{cls_name}.")
        end
        if !class_constructors.key?(cls_name)
            raise PolarRuntimeException.new("Missing constructor for class: #{cls_name}.")
        end
        cls = classes[cls_name]
        constructor = class_constructors[cls_name]
        begin
            # If constructor is a string, look it up on the class.
            fields = fields.map {|k,v| [k, Term.new(v).to_ruby]}
            if constructor.nil?
              instance = cls.new(fields)
            else
              instance = cls.send constructor, fields
            end
            cache_instance(instance, instance_id)
            instance
        rescue StandardError => e
            raise PolarRuntimeException.new("Error constructing instance of #{cls_name}: #{e}")
        end
      end

      private

      attr_reader :classes, :class_constructors

      def cache_instance(instance, id=nil)
        if id.nil?
          id = Errors.check_result(FFI.polar_get_external_id(pointer))
        end
        self.instances[id] = instance
        id
      end

      def free
        res = FFI.polar_free(pointer)
        raise Errors::FreeError if res.zero?
      end
    end

    class Query
      include Errors
      def self.from_str(query_str, polar)
        res = FFI.polar_new_query(polar.pointer, query_str)
        raise Errors::PolarError if res.null?

        new(res, polar)
      end

      def self.from_repl(polar)
        pointer = FFI.polar_query_from_repl(polar.pointer)
        if pointer.null?
          Errors.get_error
          raise RuntimeError
        end
        new(pointer, polar)
      end

      def initialize(pointer, polar)
        @pointer = pointer
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

      attr_reader :pointer, :polar, :fiber

      def free
        res = FFI.query_free(pointer)
        raise Errors::FreeError if res.zero?
      end

      def start
        @fiber = Fiber.new do
          begin
            loop do
              string = FFI.polar_query(polar.pointer, pointer)
              if string.nil?
                raise PolarRuntimeException.new Errors.get_error
              end
              event = JSON.parse(string)
              break if event == 'Done'

              event = Event.new(event)
              case event.kind
              when 'Result'
                Fiber.yield event.bindings
              when "MakeExternal"
                id = event.data["instance_id"]
                if polar.instances.key?(id)
                    raise PolarRuntimeException "Instance #{id} already registered."
                end
                cls_name = data["instance"]["tag"]
                fields = data["instance"]["fields"]["fields"]
                polar.make_external_instance(cls_name, fields, id)
              else
                p event
                raise Errors::UnhandledEventError, event.kind
              end
            end
          ensure
            free
          end
        end
      end
    end

    class Event
      attr_reader :kind

      def initialize(json)
        @kind, @data = [*json][0]
      end

      def bindings
        # Still skeptical about whether we should have a method that only works for certain types of events
        data['bindings'].sort.map { |k, v| [k, Term.new(v).to_ruby] }.to_h
      end

      private

      attr_reader :data
    end

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
          raise Errors::Unimplemented
        when 'Symbol'
          raise Errors::PolarRuntimeException
        else
          raise Errors::Unimplemented
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
  polar.query_str('f(x)')
  # polar.query_str('k(x)')

  polar.load_str('foo(1, 2); foo(3, 4); foo(5, 6);')
  if polar.query_str('foo(x, y)').to_a != [{ 'x' => 1, 'y' => 2 }, { 'x' => 3, 'y' => 4 }, { 'x' => 5, 'y' => 6 }]
    raise 'AssertionError'
  end

  t = TestClass.new
  polar.register_class(TestClass)

  # polar.load_str('external(x) := x = TestClass{};')
  # raise "AssertionError" if polar.query_str('external(x)').to_a != [{"x"=>1}]

end
