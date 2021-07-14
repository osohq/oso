# frozen_string_literal: true

module Oso
  module Polar
    # Ruby code reloaders (i.e. the one used by rails) swap out the value of
    # a constant on code changes. Because of this, we can't reliably call
    # `is_a?` on the constant that was passed to `register_class`.
    #
    # Example (where Foo is a class defined in foo.rb):
    #   > klass = Foo
    #   > Foo.new.is_a? klass
    #     => true
    #   > ... user changes foo.rb ...
    #   > Foo.new.is_a? klass
    #     => false
    #
    # To solve this, when we need to access the class (e.g. during isa), we
    # look it up using const_get, which will always return the up-to-date
    # version of the class.
    class PolarClass
      attr_reader :name

      def initialize(klass)
        @name = klass.name
      end

      def get
        Object.const_get(name)
      end
    end

    # Translate between Polar and the host language (Ruby).
    class Host # rubocop:disable Metrics/ClassLength
      protected

      # @return [FFI::Polar]
      attr_reader :ffi_polar
      # @return [Hash<String, Class>]
      attr_reader :classes
      # @return [Hash<Integer, Object>]
      attr_reader :instances

      public

      def initialize(ffi_polar)
        @ffi_polar = ffi_polar
        @classes = {}
        @instances = {}
      end

      def initialize_copy(other)
        @ffi_polar = other.ffi_polar
        @classes = other.classes.dup
        @instances = other.instances.dup
      end

      # Fetch a Ruby class from the {#classes} cache.
      #
      # @param name [String]
      # @return [Class]
      # @raise [UnregisteredClassError] if the class has not been registered.
      def get_class(name)
        raise UnregisteredClassError, name unless classes.key? name

        classes[name].get
      end

      # Store a Ruby class in the {#classes} cache.
      #
      # @param cls [Class] the class to cache.
      # @param name [String] the name to cache the class as.
      # @return [String] the name the class is cached as.
      # @raise [DuplicateClassAliasError] if attempting to register a class
      # under a previously-registered name.
      def cache_class(cls, name:)
        raise DuplicateClassAliasError.new name: name, old: get_class(name), new: cls if classes.key? name

        classes[name] = PolarClass.new(cls)
        name
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

        instance = instances[id]
        return instance.get if instance.is_a? PolarClass

        instance
      end

      # Cache a Ruby instance in the {#instances} cache, fetching a new id if
      # one isn't provided.
      #
      # @param instance [Object]
      # @param id [Integer] the instance ID. Generated via FFI if not provided.
      # @return [Integer] the instance ID.
      def cache_instance(instance, id: nil)
        id = ffi_polar.new_id if id.nil?
        instance = PolarClass.new(instance) if instance.is_a? Class
        instances[id] = instance
        id
      end

      # Construct and cache a Ruby instance.
      #
      # @param cls_name [String] name of the instance's class.
      # @param args [Array<Object>] positional args to the constructor.
      # @param kwargs [Hash<String, Object>] keyword args to the constructor.
      # @param id [Integer] the instance ID.
      # @raise [PolarRuntimeError] if instance construction fails.
      # @return [Integer] the instance ID.
      def make_instance(cls_name, args:, kwargs:, id:)
        instance = if kwargs.empty? # This check is for Ruby < 2.7.
                     get_class(cls_name).__send__(:new, *args)
                   else
                     get_class(cls_name).__send__(:new, *args, **kwargs)
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
        left_index = mro.index(get_class(left_tag))
        right_index = mro.index(get_class(right_tag))
        left_index && right_index && left_index < right_index
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
      end

      # Turn a Ruby value into a Polar term that's ready to be sent across the
      # FFI boundary.
      #
      # @param value [Object]
      # @return [Hash<String, Object>]
      def to_polar(value) # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength, Metrics/PerceivedComplexity
        value = case true # rubocop:disable Lint/LiteralAsCondition
                when value.instance_of?(TrueClass) || value.instance_of?(FalseClass)
                  { 'Boolean' => value }
                when value.instance_of?(Integer)
                  { 'Number' => { 'Integer' => value } }
                when value.instance_of?(Float)
                  if value == Float::INFINITY
                    value = 'Infinity'
                  elsif value == -Float::INFINITY
                    value = '-Infinity'
                  elsif value.nan?
                    value = 'NaN'
                  end
                  { 'Number' => { 'Float' => value } }
                when value.instance_of?(String)
                  { 'String' => value }
                when value.instance_of?(Array)
                  { 'List' => value.map { |el| to_polar(el) } }
                when value.instance_of?(Hash)
                  { 'Dictionary' => { 'fields' => value.transform_values { |v| to_polar(v) } } }
                when value.instance_of?(Predicate)
                  { 'Call' => { 'name' => value.name, 'args' => value.args.map { |el| to_polar(el) } } }
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
      def to_ruby(data) # rubocop:disable Metrics/AbcSize, Metrics/CyclomaticComplexity, Metrics/MethodLength, Metrics/PerceivedComplexity
        tag, value = data['value'].first
        case tag
        when 'String', 'Boolean'
          value
        when 'Number'
          num = value.values.first
          if value.key? 'Float'
            case num
            when 'Infinity'
              return Float::INFINITY
            when '-Infinity'
              return -Float::INFINITY
            when 'NaN'
              return Float::NAN
            else
              unless value['Float'].is_a? Float # rubocop:disable Metrics/BlockNesting
                raise PolarRuntimeError, "Expected a floating point number, got \"#{value['Float']}\""
              end
            end
          end
          num
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
