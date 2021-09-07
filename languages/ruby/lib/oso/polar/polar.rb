# frozen_string_literal: true

# Missing Ruby type.
module PolarBoolean; end
# Monkey-patch Ruby true type.
class TrueClass; include PolarBoolean; end
# Monkey-patch Ruby false type.
class FalseClass; include PolarBoolean; end

# https://github.com/ruby/ruby/blob/bb9ecd026a6cadd5d0f85ac061649216806ed935/lib/bundler/vendor/thor/lib/thor/shell/color.rb#L99-L105
def supports_color
  $stdout.tty? && $stderr.tty? && ENV['NO_COLOR'].nil?
end

if supports_color
  RESET = "\x1b[0m"
  FG_BLUE = "\x1b[34m"
  FG_RED = "\x1b[31m"
else
  RESET = ''
  FG_BLUE = ''
  FG_RED = ''
end

def print_error(error)
  warn FG_RED + error.class.name.split('::').last + RESET
  warn error.message
end

module Oso
  module Polar
    # Create and manage an instance of the Polar runtime.
    class Polar # rubocop:disable Metrics/ClassLength
      # @return [Host]
      attr_reader :host

      def initialize
        @ffi_polar = FFI::Polar.create
        @host = Host.new(ffi_polar)
        @ffi_polar.enrich_message = @host.method(:enrich_message)

        # Register global constants.
        register_constant nil, name: 'nil'

        # Register built-in classes.
        register_class PolarBoolean, name: 'Boolean'
        register_class Integer
        register_class Float
        register_class Array, name: 'List'
        register_class Hash, name: 'Dictionary'
        register_class String
      end

      def ffi
        @ffi_polar
      end

      # get the (maybe user-supplied) name of a class.
      # kind of a hack because of class autoreloading.
      def get_class_name(klass) # rubocop:disable Metrics/AbcSize
        if host.types.key? klass
          host.types[klass].name
        elsif host.types.key? klass.name
          host.types[klass.name].name
        else
          rec = host.types.values.find { |v| v.klass.get == klass }
          raise "Unknown class `#{klass}`" if rec.nil?

          host.types[klass] = rec
          rec.name
        end
      end

      def type_constraint(var, cls)
        Expression.new(
          'And',
          [Expression.new('Isa', [var, Pattern.new(get_class_name(cls), {})])]
        )
      end

      # Returns a query for the resources belonging to +cls+ that +actor+
      # is allowed to perform +action+ on.
      def authorized_query(actr, actn, cls) # rubocop:disable Metrics/MethodLength
        rsrc = Variable.new 'resource'

        results = query_rule(
          'allow',
          actr,
          actn,
          rsrc,
          bindings: { 'resource' => type_constraint(rsrc, cls) },
          accept_expression: true
        )

        results = results.each_with_object([]) do |result, out|
          result.each do |key, val|
            out.push({ 'bindings' => { key => host.to_polar(val) } })
          end
        end

        ::Oso::Polar::DataFiltering::FilterPlan
          .parse(self, results, get_class_name(cls))
          .build_query
      end

      def authorized_resources(actr, actn, cls)
        q = authorized_query actr, actn, cls
        return [] if q.nil?

        host.types[get_class_name cls].exec_query[q]
      end

      # Clear all rules and rule sources from the current Polar instance
      #
      # @return [self] for chaining.
      def clear_rules
        ffi_polar.clear_rules
        self
      end

      # Load a Polar policy file.
      #
      # @param name [String]
      # @raise [PolarFileExtensionError] if provided filename has invalid extension.
      # @raise [PolarFileNotFoundError] if provided filename does not exist.
      # @return [self] for chaining.
      def load_file(name)
        raise PolarFileExtensionError, name unless File.extname(name) == '.polar'

        file_data = File.open(name, &:read)
        load_str(file_data, filename: name)
      rescue Errno::ENOENT
        raise PolarFileNotFoundError, name
      end

      # Load a Polar string into the KB.
      #
      # @param str [String] Polar string to load.
      # @param filename [String] Name of Polar source file.
      # @raise [NullByteInPolarFileError] if str includes a non-terminating null byte.
      # @raise [InlineQueryFailedError] on the first failed inline query.
      # @raise [Error] if any of the FFI calls raise one.
      # @return [self] for chaining.
      def load_str(str, filename: nil) # rubocop:disable Metrics/AbcSize
        raise NullByteInPolarFileError if str.chomp("\0").include?("\0")

        host.register_mros

        ffi_polar.load(str, filename: filename)
        loop do
          next_query = ffi_polar.next_inline_query
          break if next_query.nil?

          raise InlineQueryFailedError, next_query.source if Query.new(next_query, host: host).first.nil?
        end

        self
      end

      # Query for a Polar predicate or string.
      #
      # @overload query(query)
      #   @param query [String]
      #   @return [Enumerator] of resulting bindings
      #   @raise [Error] if the FFI call raises one.
      # @overload query(query)
      #   @param query [Predicate]
      #   @return [Enumerator] of resulting bindings
      #   @raise [Error] if the FFI call raises one.
      def query(query, host: self.host.dup, bindings: {})
        case query
        when String
          ffi_query = ffi_polar.new_query_from_str(query)
        when Predicate
          ffi_query = ffi_polar.new_query_from_term(host.to_polar(query))
        else
          raise InvalidQueryTypeError
        end
        Query.new(ffi_query, host: host, bindings: bindings)
      end

      # Query for a rule.
      #
      # @param name [String]
      # @param args [Array<Object>]
      # @return [Enumerator] of resulting bindings
      # @raise [Error] if the FFI call raises one.
      def query_rule(name, *args, accept_expression: false, bindings: {})
        host = self.host.dup
        host.accept_expression = accept_expression
        query(Predicate.new(name, args: args), host: host, bindings: bindings)
      end

      # Query for a rule, returning true if it has any results.
      #
      # @param name [String]
      # @param args [Array<Object>]
      # @return [Boolean] indicating whether the query found at least one result.
      # @raise [Error] if the FFI call raises one.
      def query_rule_once(name, *args)
        query_rule(name, *args).any?
      end

      # Register a Ruby class with Polar.
      #
      # @param cls [Class] the class to register.
      # @param name [String] the name to register the class as. Defaults to the name of the class.
      # @raise [DuplicateClassAliasError] if attempting to register a class
      # under a previously-registered name.
      # @raise [FFI::Error] if the FFI call returns an error.
      # @return [self] for chaining.
      def register_class(cls, name: nil, fields: nil, combine_query: nil, build_query: nil, exec_query: nil) # rubocop:disable Metrics/ParameterLists
        name = host.cache_class(
          cls,
          name: name || cls.name,
          fields: fields,
          build_query: build_query || maybe_mtd(cls, :build_query),
          combine_query: combine_query || maybe_mtd(cls, :combine_query),
          exec_query: exec_query || maybe_mtd(cls, :exec_query)
        )
        register_constant(cls, name: name)
      end

      # Register a Ruby object with Polar.
      #
      # @param value [Object] the object to register.
      # @param name [String] the name to register the object as.
      # @return [self] for chaining.
      # @raise [FFI::Error] if the FFI call returns an error.
      def register_constant(value, name:)
        ffi_polar.register_constant(host.to_polar(value), name: name)
        self
      end

      # Start a REPL session.
      #
      # @param files [Array<String>]
      # @raise [Error] if the FFI call raises one.
      def repl(files = [])
        files.map { |f| load_file(f) }
        prompt = "#{FG_BLUE}query>#{RESET} "
        # Try loading the readline module from the Ruby stdlib. If we get a
        # LoadError, fall back to the standard REPL with no readline support.
        require 'readline'
        repl_readline(prompt)
      rescue LoadError
        repl_standard(prompt)
      end

      private

      def maybe_mtd(cls, mtd)
        cls.respond_to?(mtd) && cls.method(mtd) || nil
      end

      # @return [FFI::Polar]
      attr_reader :ffi_polar

      # The R and L in REPL for systems where readline is available.
      def repl_readline(prompt)
        while (buf = Readline.readline(prompt, true))
          if /^\s*$/ =~ buf # Don't add empty entries to history.
            Readline::HISTORY.pop
            next
          end
          process_line(buf)
        end
      rescue Interrupt # rubocop:disable Lint/SuppressedException
      end

      # The R and L in REPL for systems where readline is not available.
      def repl_standard(prompt)
        loop do
          puts prompt
          begin
            buf = $stdin.readline
          rescue EOFError
            return
          end
          process_line(buf)
        end
      rescue Interrupt # rubocop:disable Lint/SuppressedException
      end

      # Process a line of user input in the REPL.
      #
      # @param buf [String]
      def process_line(buf) # rubocop:disable Metrics/MethodLength, Metrics/PerceivedComplexity, Metrics/AbcSize
        query = buf.chomp.chomp(';')
        begin
          ffi_query = ffi_polar.new_query_from_str(query)
        rescue ParseError => e
          print_error(e)
          return
        end

        begin
          results = Query.new(ffi_query, host: host).to_a
        rescue PolarRuntimeError => e
          print_error(e)
          return
        end

        if results.empty?
          puts false
        else
          results.each do |result|
            if result.empty?
              puts true
            else
              result.each do |variable, value|
                puts "#{variable} = #{value.inspect}"
              end
            end
          end
        end
      end
    end
  end
end
