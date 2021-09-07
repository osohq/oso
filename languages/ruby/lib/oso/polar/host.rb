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
      attr_reader :name, :anon_class

      def initialize(klass)
        @name = klass.name
        # If the class doesn't have a name, it is anonymous, meaning we should
        # actually store it directly
        @anon_class = klass if klass.name.nil?
      end

      def get
        return anon_class if anon_class

        Object.const_get(name)
      end
    end

    # For holding type metadata: name, fields, etc.
    class UserType
      attr_reader :name, :klass, :id, :fields, :build_query, :combine_query, :exec_query

      def initialize(name:, klass:, id:, fields:, build_query:, combine_query:, exec_query:) # rubocop:disable Metrics/ParameterLists
        @name = name
        @klass = klass
        @id = id
        # accept symbol keys
        @fields = fields.each_with_object({}) { |kv, o| o[kv[0].to_s] = kv[1] }
        @build_query = build_query
        @combine_query = combine_query
        @exec_query = exec_query
      end
    end

    # Translate between Polar and the host language (Ruby).
    class Host # rubocop:disable Metrics/ClassLength
      # @return [Hash<String, UserType>]
      attr_reader :types

      protected

      # @return [FFI::Polar]
      attr_reader :ffi_polar
      # @return [Hash<Integer, Object>]
      attr_reader :instances
      # @return [Boolean]
      attr_reader :accept_expression

      public

      attr_writer :accept_expression

      def initialize(ffi_polar)
        @ffi_polar = ffi_polar
        @types = {}
        @instances = {}
        @accept_expression = false
      end

      def initialize_copy(other)
        @ffi_polar = other.ffi_polar
        @types = other.types.dup
        @instances = other.instances.dup
      end

      # Fetch a Ruby class from the {#types} cache.
      #
      # @param name [String]
      # @return [Class]
      # @raise [UnregisteredClassError] if the class has not been registered.
      def get_class(name)
        raise UnregisteredClassError, name unless types.key? name

        types[name].klass.get
      end

      # Store a Ruby class in the {#types} cache.
      #
      # @param cls [Class] the class to cache.
      # @param name [String] the name to cache the class as.
      # @return [String] the name the class is cached as.
      # @raise [DuplicateClassAliasError] if attempting to register a class
      # under a previously-registered name.
      def cache_class(cls, name:, fields:, build_query:, combine_query:, exec_query:) # rubocop:disable Metrics/ParameterLists, Metrics/MethodLength
        raise DuplicateClassAliasError.new name: name, old: get_class(name), new: cls if types.key? name

        types[name] = types[cls] = UserType.new(
          name: name,
          klass: PolarClass.new(cls),
          id: cache_instance(cls),
          fields: fields || {},
          combine_query: combine_query,
          exec_query: exec_query,
          build_query: build_query
        )
        name
      end

      def register_mros # rubocop:disable Metrics/AbcSize
        types.values.uniq.each do |typ|
          mro = []
          typ.klass.get.ancestors.each do |a|
            mro.push(types[a].id) if types.key?(a)
          end
          ffi_polar.register_mro(typ.name, mro)
        end
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
        # Save the instance as a PolarClass if it is a non-anonymous class
        instance = PolarClass.new(instance) if instance.is_a?(Class)
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

      OPS = {
        'Lt' => :<,
        'Gt' => :>,
        'Eq' => :==,
        'Geq' => :>=,
        'Leq' => :<=,
        'Neq' => :!=
      }.freeze

      # Compare two values
      #
      # @param op [String] operation to perform.
      # @param args [Array<Object>] left and right args to operation.
      # @raise [PolarRuntimeError] if operation fails or is unsupported.
      # @return [Boolean]
      def operator(operation, args)
        left, right = args
        op = OPS[operation]
        raise PolarRuntimeError, "Unsupported external operation '#{left.class} #{operation} #{right.class}'" if op.nil?

        begin
          left.__send__ op, right
        rescue StandardError
          raise PolarRuntimeError, "External operation '#{left.class} #{operation} #{right.class}' failed."
        end
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

      def subclass?(left_tag:, right_tag:)
        get_class(left_tag) <= get_class(right_tag)
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

      def serialize_types # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        polar_types = {}
        types.values.uniq.each do |typ|
          tag = typ.name
          fields = typ.fields
          field_types = {}
          fields.each do |k, v|
            field_types[k] =
              if v.is_a? ::Oso::Polar::DataFiltering::Relation
                {
                  'Relation' => {
                    'kind' => v.kind,
                    'other_class_tag' => v.other_type,
                    'my_field' => v.my_field,
                    'other_field' => v.other_field
                  }
                }
              else
                { 'Base' => { 'class_tag' => types[v].name } }
              end
          end
          polar_types[tag] = field_types
        end
        polar_types
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
                  { 'Variable' => value.name }
                when value.instance_of?(Expression)
                  { 'Expression' => { 'operator' => value.operator, 'args' => value.args.map { |el| to_polar(el) } } }
                when value.instance_of?(Pattern)
                  dict = to_polar(value.fields)['value']
                  if value.tag.nil?
                    { 'Pattern' => dict }
                  else
                    { 'Pattern' => { 'Instance' => { 'tag' => value.tag, 'fields' => dict['Dictionary'] } } }
                  end
                else
                  instance_id = nil
                  instance_id = types[value].id if value.is_a?(Class) && types.key?(value)
                  { 'ExternalInstance' => { 'instance_id' => cache_instance(value, id: instance_id), 'repr' => nil } }
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
          Variable.new(value)
        when 'Expression'
          raise UnexpectedPolarTypeError, tag unless accept_expression

          args = value['args'].map { |a| to_ruby(a) }
          Expression.new(value['operator'], args)
        when 'Pattern'
          case value.keys.first
          when 'Instance'
            tag = value.values.first['tag']
            fields = value.values.first['fields']['fields'].transform_values { |v| to_ruby(v) }
            Pattern.new(tag, fields)
          when 'Dictionary'
            fields = value.values.first['fields'].transform_values { |v| to_ruby(v) }
            Pattern.new(nil, fields)
          else
            raise UnexpectedPolarTypeError, "#{value.keys.first} variant of Pattern"
          end
        else
          raise UnexpectedPolarTypeError, tag
        end
      end

      def enrich_message(msg)
        msg.gsub(/\^\{id: ([0-9]+)\}/) do
          get_instance(Regexp.last_match[1].to_i).to_s
        end
      end
    end
  end
end
