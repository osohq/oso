# frozen_string_literal: true

module Oso
  module Polar
    # Translate between Polar and the host language (Ruby).
    class Host # rubocop:disable Metrics/ClassLength
      protected

      # @return [FFI::Polar]
      attr_reader :ffi_polar
      # @return [Hash<String, Class>]
      attr_reader :classes
      # @return [Hash<String, Object>]
      attr_reader :constructors
      # @return [Hash<Integer, Object>]
      attr_reader :instances

      public

      def initialize(ffi_polar)
        @ffi_polar = ffi_polar
        @classes = {}
        @constructors = {}
        @instances = {}
      end

      def initialize_copy(other)
        @ffi_polar = other.ffi_polar
        @classes = other.classes.dup
        @constructors = other.constructors.dup
        @instances = other.instances.dup
      end

      # Fetch a Ruby class from the {#classes} cache.
      #
      # @param name [String]
      # @return [Class]
      # @raise [UnregisteredClassError] if the class has not been registered.
      def get_class(name)
        raise UnregisteredClassError, name unless classes.key? name

        classes[name]
      end

      # Store a Ruby class in the {#classes} cache.
      #
      # @param cls [Class] the class to cache
      # @param name [String] the name to cache the class as. Defaults to the name of the class.
      # @param constructor [Proc] optional custom constructor function. Defaults to the :new method.
      # @return [String] the name the class is cached as.
      # @raise [UnregisteredClassError] if the class has not been registered.
      def cache_class(cls, name:, constructor:) # rubocop:disable Metrics/MethodLength
        name = cls.name if name.nil?
        raise DuplicateClassAliasError, name: name, old: get_class(name), new: cls if classes.key? name

        classes[name] = cls
        if constructor.nil?
          constructors[name] = :new
        elsif constructor.respond_to? :call
          constructors[name] = constructor
        else
          raise InvalidConstructorError
        end
        name
      end

      # Fetch a constructor from the {#constructors} cache.
      #
      # @param name [String]
      # @return [Symbol] if constructor is the default of `:new`.
      # @return [Proc] if a custom constructor was registered.
      # @raise [UnregisteredConstructorError] if the constructor has not been registered.
      def get_constructor(name)
        raise MissingConstructorError, name unless constructors.key? name

        constructors[name]
      end

      # Check if an instance exists in the {#instances} cache.
      #
      # @param id [Integer]
      # @return [Boolean]
      def instance?(id)
        case id
        when Integer
          instances.key? id
        else
          instances.value? id
        end
      end

      # Fetch a Ruby instance from the {#instances} cache.
      #
      # @param id [Integer]
      # @return [Object]
      # @raise [UnregisteredInstanceError] if the ID has not been registered.
      def get_instance(id)
        raise UnregisteredInstanceError, id unless instance? id

        instances[id]
      end

      # Cache a Ruby instance in the {#instances} cache, fetching a {#new_id}
      # if one isn't provided.
      #
      # @param instance [Object]
      # @param id [Integer]
      # @return [Integer]
      def cache_instance(instance, id: nil)
        id = ffi_polar.new_id if id.nil?
        instances[id] = instance
        id
      end

      # Construct and cache a Ruby instance.
      #
      # @param cls_name [String]
      # @param args [Array<Object>]
      # @param kwargs [Hash<String, Object>]
      # @param id [Integer]
      # @raise [PolarRuntimeError] if instance construction fails.
      def make_instance(cls_name, args:, kwargs:, id:) # rubocop:disable Metrics/MethodLength
        constructor = get_constructor(cls_name)
        instance = if constructor == :new
                     if kwargs.empty?
                       get_class(cls_name).__send__(:new, *args)
                     else
                       get_class(cls_name).__send__(:new, *args, **kwargs)
                     end
                   elsif kwargs.empty?
                     constructor.call(*args)
                   else
                     constructor.call(*args, **kwargs)
                   end
        cache_instance(instance, id: id)
      rescue StandardError => e
        raise PolarRuntimeError, "Error constructing instance of #{cls_name}: #{e}"
      end

      # Check if the left class is more specific than the right class
      # with respect to the given instance.
      #
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

      # Check if instance is an instance of class.
      #
      # @param instance [Hash<String, Object>]
      # @param class_tag [String]
      # @return [Boolean]
      def isa?(instance, class_tag:)
        instance = to_ruby(instance)
        cls = get_class(class_tag)
        instance.is_a? cls
      rescue PolarRuntimeError
        false
      end

      # Check if two instances unify
      #
      # @param left_instance_id [Integer]
      # @param right_instance_id [Integer]
      # @return [Boolean]
      def unify?(left_instance_id, right_instance_id)
        left_instance = get_instance(left_instance_id)
        right_instance = get_instance(right_instance_id)
        left_instance == right_instance
      rescue PolarRuntimeError
        false
      end

      # Turn a Ruby value into a Polar term that's ready to be sent across the
      # FFI boundary.
      #
      # @param value [Object]
      # @return [Hash<String, Object>]
      def to_polar_term(value) # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength
        value = case true # rubocop:disable Lint/LiteralAsCondition
                when value.instance_of?(TrueClass) || value.instance_of?(FalseClass)
                  { 'Boolean' => value }
                when value.instance_of?(Integer)
                  { 'Number' => { 'Integer' => value } }
                when value.instance_of?(Float)
                  { 'Number' => { 'Float' => value } }
                when value.instance_of?(String)
                  { 'String' => value }
                when value.instance_of?(Array)
                  { 'List' => value.map { |el| to_polar_term(el) } }
                when value.instance_of?(Hash)
                  { 'Dictionary' => { 'fields' => value.transform_values { |v| to_polar_term(v) } } }
                when value.instance_of?(Predicate)
                  { 'Call' => { 'name' => value.name, 'args' => value.args.map { |el| to_polar_term(el) } } }
                when value.instance_of?(Variable)
                  # This is supported so that we can query for unbound variables
                  { 'Variable' => value }
                else
                  { 'ExternalInstance' => { 'instance_id' => cache_instance(value), 'repr' => value.to_s } }
                end
        { 'value' => value }
      end

      # Turn a Polar term passed across the FFI boundary into a Ruby value.
      #
      # @param data [Hash<String, Object>]
      # @option data [Integer] :id
      # @option data [Integer] :offset Character offset of the term in its source string.
      # @option data [Hash<String, Object>] :value
      # @return [Object]
      # @raise [UnexpectedPolarTypeError] if type cannot be converted to Ruby.
      def to_ruby(data) # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength
        tag, value = data['value'].first
        case tag
        when 'String', 'Boolean'
          value
        when 'Number'
          value.values.first
        when 'List'
          value.map { |el| to_ruby(el) }
        when 'Dictionary'
          value['fields'].transform_values { |v| to_ruby(v) }
        when 'ExternalInstance'
          get_instance(value['instance_id'])
        when 'Call'
          Predicate.new(value['name'], args: value['args'].map { |a| to_ruby(a) })
        when 'Variable'
          Variable.new(value['name'])
        else
          raise UnexpectedPolarTypeError, tag
        end
      end
    end
  end
end
