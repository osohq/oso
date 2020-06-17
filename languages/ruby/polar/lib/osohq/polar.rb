# frozen_string_literal: true

require 'json'
require 'set'

require 'osohq/polar/version'
require 'osohq/polar/ffi'
require 'osohq/polar/errors'

module Osohq
  module Polar
    # TODO(gj): document
    class Polar
      attr_reader :instances, :calls

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
      def load(str)
        raise NullByteInPolarFileError if str.chomp("\0").include?("\0")

        ffi_instance.load(str)
        loop do
          next_query = ffi_instance.next_inline_query
          break if next_query.nil?

          begin
            Query.new(next_query, polar: self).results.next
          rescue StopIteration
            raise InlineQueryFailedError
          end
        end
      end

      def query_str(str)
        load_queued_files
        query_ffi_instance = ffi_instance.new_query_from_str(str)
        Query.new(query_ffi_instance, polar: self).results
      end

      # @param pred [Predicate]
      def query_pred(pred)
        load_queued_files
        query_ffi_instance = ffi_instance.new_query_from_term(to_polar_term(pred))
        Query.new(query_ffi_instance, polar: self).results
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

      # @param cls [Class]
      # @param from_polar [Object]
      def register_class(cls, &from_polar)
        classes[cls.name] = cls
        from_polar = :new if from_polar.nil?
        constructors[cls.name] = from_polar
      end

      # @param method [#to_sym]
      # @param args [Array<Hash>]
      # @param call_id [Integer]
      # @param instance_id [Integer]
      def register_call(method, args:, call_id:, instance_id:)
        args = args.map { |a| to_ruby(Term.new(a)) }
        instance = get_instance(instance_id)
        result = instance.__send__(method, *args)
        result = [result].to_enum unless result.instance_of? Enumerator # Call must be a generator.
        calls[call_id] = result.lazy
      rescue ArgumentError, NoMethodError
        raise InvalidCallError
      end

      # @param cls_name [String]
      # @param fields [Hash<String, Hash>]
      # @param id [Integer]
      def make_instance(cls_name, fields:, id: nil)
        raise MissingConstructorError, cls_name unless constructors.key?(cls_name)

        fields = Hash[fields.map { |k, v| [k.to_sym, to_ruby(Term.new(v))] }]
        instance = if constructors[cls_name] == :new
                     get_class(cls_name).__send__(:new, **fields)
                   else
                     constructors[cls_name].call(**fields)
                   end
        cache_instance(instance, id: id)
      rescue StandardError => e
        raise PolarRuntimeError, "Error constructing instance of #{cls_name}: #{e}"
      end

      # Clear the KB but retain all registered classes and constructors.
      def clear
        # TODO(gj): Should we clear out instance + call caches as well?
        @ffi_instance = FFI::Polar.create
      end

      # Enqueue a Polar policy file for loading into the KB.
      # @param file [String]
      def load_file(file)
        unless ['.pol', '.polar'].include? File.extname(file)
          raise PolarRuntimeError, 'Polar files must have .pol or .polar extension.'
        end
        raise PolarRuntimeError, "Could not find file: #{file}" unless File.file?(file)

        load_queue << file
      end

      # @param instance_id [Integer]
      # @param left_tag [String]
      # @param right_tag [String]
      # @return [Boolean]
      def subspecializer?(instance_id, left_tag:, right_tag:)
        mro = get_instance(instance_id).class.ancestors
        mro.index(get_class(left_tag)) < mro.index(get_class(right_tag))
      rescue StandardError
        false
      end

      # @param instance_id [Integer]
      # @param class_tag [String]
      def isa?(instance_id, class_tag:)
        cls = get_class(class_tag)
        instance = get_instance(instance_id)
        instance.is_a? cls
      rescue PolarRuntimeError
        false
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

      # @param term [Term]
      # @return [Object]
      def to_ruby(term)
        tag = term.tag
        value = term.value
        case tag
        when 'Integer', 'String', 'Boolean'
          value
        when 'List'
          value.map { |el| to_ruby(Term.new(el)) }
        when 'Dictionary'
          value['fields'].transform_values { |v| to_ruby(Term.new(v)) }
        when 'ExternalInstance'
          get_instance(value['instance_id'])
        when 'InstanceLiteral'
          # TODO(gj): Should InstanceLiterals ever be making it to Ruby?
          # convert instance literals to external instances
          cls_name = value['tag']
          fields = value['fields']['fields']
          make_instance(cls_name, fields: fields)
        when 'Call'
          Predicate.new(value['name'], args: value['args'].map { |a| to_ruby(Term.new(a)) })
        when 'Symbol'
          raise PolarRuntimeError
        else
          raise 'Unimplemented!'
        end
      end

      private

      #### PRIVATE FIELDS + METHODS ####

      attr_reader :ffi_instance, :classes, :constructors, :load_queue

      # @param instance [Object]
      # @param id [Integer]
      # @return [Integer]
      def cache_instance(instance, id: nil)
        id = ffi_instance.new_id if id.nil?
        instances[id] = instance
        id
      end

      # @param id [Integer]
      # @return [Object]
      # @raise [UnregisteredInstanceError] if the ID has not been registered.
      def get_instance(id)
        raise UnregisteredInstanceError, id unless instances.key? id

        instances[id]
      end

      # @param name [String]
      # @return [Class]
      # @raise [UnregisteredClassError] if the class has not been registered.
      def get_class(name)
        raise UnregisteredClassError, name unless classes.key? name

        classes[name]
      end

      def load_queued_files
        instances.clear
        load_queue.reject! do |file|
          File.open(file) { |f| load(f.read) }
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
        unless polar.calls.key?(call_id)
          polar.register_call(method, args: args, call_id: call_id, instance_id: instance_id)
        end

        # Return the next result of the call.
        begin
          value = polar.calls[call_id].next
          stringified = JSON.dump(polar.to_polar_term(value))
          call_result(stringified, call_id: call_id)
        rescue StopIteration
          call_result(nil, call_id: call_id)
        end
      rescue InvalidCallError
        call_result(nil, call_id: call_id)
        # @TODO: polar line numbers in errors once polar errors are better.
        # raise PolarRuntimeError(f"Error calling {attribute}")
      end

      private

      attr_reader :ffi_instance, :polar, :fiber

      def start
        @fiber = Fiber.new do
          loop do
            event = ffi_instance.next_event
            case event.kind
            when 'Done'
              break
            when 'Result'
              Fiber.yield(event.data['bindings'].transform_values { |v| polar.to_ruby(Term.new(v)) })
            when 'MakeExternal'
              id = event.data['instance_id']
              raise DuplicateInstanceRegistrationError, id if polar.instances.key?(id)

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

    # TODO(gj): document
    class QueryEvent
      attr_reader :kind, :data

      def initialize(event_data)
        event_data = { event_data => nil } if event_data == 'Done'
        @kind, @data = event_data.first
      end
    end

    # Polar term.
    class Term
      attr_reader :value, :tag

      # @param data [Hash<String, Object>]
      # @option data [Integer] :id
      # @option data [Integer] :offset Character offset of the term in its source string.
      # @option data [Hash<String, Object>] :value
      def initialize(data)
        @id = data['id']
        @offset = data['offset']
        @tag, @value = data['value'].first
      end
    end

    # Polar predicate.
    class Predicate
      attr_reader :name, :args

      # @param name [String]
      # @param args [Array]
      def initialize(name, args:)
        @name = name
        @args = args
      end

      def ==(other)
        name == other.name && args == other.args
      end

      alias eql? ==
    end

    # Polar variable.
    class Variable
      attr_reader :name

      # @param name [String]
      def initialize(name)
        @name = name
      end

      def to_s
        name
      end
    end
  end
end

Osohq::Polar::Polar.new.tap do |polar|
  polar.load('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  puts 'f(x)', polar.query_str('f(x)').to_a
  puts 'k(x)', polar.query_str('k(x)').to_a

  polar.load('foo(1, 2); foo(3, 4); foo(5, 6);')
  if polar.query_str('foo(x, y)').to_a != [{ 'x' => 1, 'y' => 2 }, { 'x' => 3, 'y' => 4 }, { 'x' => 5, 'y' => 6 }]
    raise 'AssertionError'
  end

  class TestClass
    def my_method
      1
    end
  end

  polar.register_class(TestClass)

  polar.load('external(x, 3) := x = TestClass{}.my_method;')
  results = polar.query_str('external(1, x)')
  p results.next

  # polar.load('testDebug() := debug(), foo(x, y), k(y);')
  # polar.query_str('testDebug()').next
end
