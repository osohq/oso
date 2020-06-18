# frozen_string_literal: true

require 'json'
require 'set'

module Osohq
  module Polar
    # Create and manage an instance of the Polar runtime.
    class Polar
      def initialize
        @ffi_instance = FFI::Polar.create
        @calls = {}
        @classes = {}
        @constructors = {}
        @instances = {}
        @load_queue = Set.new
      end

      # Load a Polar string into the KB.
      #
      # @param str [String] Polar string to load.
      def load_str(str)
        raise NullByteInPolarFileError if str.chomp("\0").include?("\0")

        ffi_instance.load_str(str)
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

      # @param name [String]
      # @param args [Array<Object>]
      def query_pred(name, args:)
        clear_query_state
        load_queued_files
        pred = Predicate.new(name, args: args)
        query_ffi_instance = ffi_instance.new_query_from_term(to_polar_term(pred))
        Query.new(query_ffi_instance, polar: self).results
      end

      def repl
        clear_query_state
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

      # @return [Integer]
      def new_id
        ffi_instance.new_id
      end

      # @param id [Integer]
      # @return [Boolean]
      def instance?(id)
        instances.key? id
      end

      # @param method [#to_sym]
      # @param args [Array<Hash>]
      # @param call_id [Integer]
      # @param instance_id [Integer]
      def register_call(method, args:, call_id:, instance_id:)
        return if calls.key?(call_id)

        args = args.map { |a| to_ruby(a) }
        instance = get_instance(instance_id)
        result = instance.__send__(method, *args)
        result = [result].to_enum unless result.is_a? Enumerator # Call must be a generator.
        calls[call_id] = result.lazy
      rescue ArgumentError, NoMethodError
        raise InvalidCallError
      end

      # @param cls_name [String]
      # @param fields [Hash<String, Hash>]
      # @param id [Integer]
      def make_instance(cls_name, fields:, id:)
        constructor = get_constructor(cls_name)
        fields = Hash[fields.map { |k, v| [k.to_sym, to_ruby(v)] }]
        instance = if constructor == :new
                     get_class(cls_name).__send__(:new, **fields)
                   else
                     constructor.call(**fields)
                   end
        cache_instance(instance, id: id)
      rescue StandardError => e
        raise PolarRuntimeError, "Error constructing instance of #{cls_name}: #{e}"
      end

      # Clear the KB but retain all registered classes and constructors.
      def clear
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
      # @return [Boolean]
      def isa?(instance_id, class_tag:)
        instance = get_instance(instance_id)
        cls = get_class(class_tag)
        instance.is_a? cls
      rescue PolarRuntimeError
        false
      end

      # Turn a Ruby value into a Polar term.
      # @param x [Object]
      # @return [Hash<String, Object>]
      def to_polar_term(x)
        val = case true
              when x.instance_of?(TrueClass) || x.instance_of?(FalseClass)
                { 'Boolean' => x }
              when x.instance_of?(Integer)
                { 'Integer' => x }
              when x.instance_of?(String)
                { 'String' => x }
              when x.instance_of?(Array)
                { 'List' => x.map { |el| to_polar_term(el) } }
              when x.instance_of?(Hash)
                { 'Dictionary' => { 'fields' => x.transform_values { |v| to_polar_term(v) } } }
              when x.instance_of?(Predicate)
                { 'Call' => { 'name' => x.name, 'args' => x.args.map { |el| to_polar_term(el) } } }
              when x.instance_of?(Variable)
                # This is supported so that we can query for unbound variables
                { 'Symbol' => x }
              else
                { 'ExternalInstance' => { 'instance_id' => cache_instance(x) } }
              end
        { 'id' => 0, 'offset' => 0, 'value' => val }
      end

      # @param id [Integer]
      # @return [Hash]
      # @raise [StopIteration] if the call has been exhausted.
      def next_call_result(id)
        to_polar_term(calls[id].next)
      end

      # @param data [Hash<String, Object>]
      # @option data [Integer] :id
      # @option data [Integer] :offset Character offset of the term in its source string.
      # @option data [Hash<String, Object>] :value
      # @return [Object]
      # @raise [UnexpectedPolarTypeError] if type cannot be converted to Ruby.
      def to_ruby(data)
        id = data['id']
        offset = data['offset']
        tag, value = data['value'].first
        case tag
        when 'Integer', 'String', 'Boolean'
          value
        when 'List'
          value.map { |el| to_ruby(el) }
        when 'Dictionary'
          value['fields'].transform_values { |v| to_ruby(v) }
        when 'ExternalInstance'
          get_instance(value['instance_id'])
        when 'Call'
          Predicate.new(value['name'], args: value['args'].map { |a| to_ruby(a) })
        else
          raise UnexpectedPolarTypeError, tag
        end
      end

      private

      attr_reader :calls, :classes, :constructors, :ffi_instance, :instances, :load_queue

      # Clear the instance and call caches
      def clear_query_state
        calls.clear
        instances.clear
      end

      def query_str(str)
        clear_query_state
        load_queued_files
        query_ffi_instance = ffi_instance.new_query_from_str(str)
        Query.new(query_ffi_instance, polar: self).results
      end

      # @param instance [Object]
      # @param id [Integer]
      # @return [Integer]
      def cache_instance(instance, id: nil)
        id = new_id if id.nil?
        instances[id] = instance
        id
      end

      # @param id [Integer]
      # @return [Object]
      # @raise [UnregisteredInstanceError] if the ID has not been registered.
      def get_instance(id)
        raise UnregisteredInstanceError, id unless instance? id

        instances[id]
      end

      def load_queued_files
        load_queue.reject! do |file|
          File.open(file) { |f| load_str(f.read) }
          true
        end
      end

      # @param name [String]
      # @return [Class]
      # @raise [UnregisteredClassError] if the class has not been registered.
      def get_class(name)
        raise UnregisteredClassError, name unless classes.key? name

        classes[name]
      end

      # @param name [String]
      # @return [Symbol] if constructor is the default of `:new`.
      # @return [Proc] if a custom constructor was registered.
      # @raise [UnregisteredConstructorError] if the constructor has not been registered.
      def get_constructor(name)
        raise MissingConstructorError, name unless constructors.key? name

        constructors[name]
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
