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
        # @ffi_polar.set_message_enricher { |msg| @host.enrich_message(msg) }
        @polar_roles_enabled = false

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

      def enable_roles # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        return if polar_roles_enabled

        roles_helper = Class.new do
          def self.join(separator, left, right)
            [left, right].join(separator)
          end
        end
        register_constant(roles_helper, name: '__oso_internal_roles_helpers__')
        ffi_polar.enable_roles
        self.polar_roles_enabled = true

        # validate config
        validation_query_results = []
        loop do
          query = ffi_polar.next_inline_query
          break if query.nil?

          new_host = host.dup
          new_host.accept_expression = true
          results = Query.new(query, host: new_host).to_a
          raise InlineQueryFailedError, query.source if results.empty?

          validation_query_results.push results
        end

        # turn bindings back into polar
        validation_query_results = validation_query_results.map do |results|
          results.map do |result|
            { 'bindings' => result.transform_values { |v| host.to_polar(v) } }
          end
        end

        ffi_polar.validate_roles_config(validation_query_results)
      end

      # Clear all rules and rule sources from the current Polar instance
      #
      # @return [self] for chaining.
      def clear_rules
        ffi_polar.clear_rules
        ffi_polar.enable_roles if polar_roles_enabled
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
      def load_str(str, filename: nil) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        raise NullByteInPolarFileError if str.chomp("\0").include?("\0")

        ffi_polar.load(str, filename: filename)
        loop do
          next_query = ffi_polar.next_inline_query
          break if next_query.nil?

          raise InlineQueryFailedError, next_query.source if Query.new(next_query, host: host).first.nil?
        end

        # If roles are enabled, re-validate config when new rules are loaded.
        if polar_roles_enabled
          self.polar_roles_enabled = false
          enable_roles
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
      def query(query)
        new_host = host.dup
        case query
        when String
          ffi_query = ffi_polar.new_query_from_str(query)
        when Predicate
          ffi_query = ffi_polar.new_query_from_term(new_host.to_polar(query))
        else
          raise InvalidQueryTypeError
        end
        Query.new(ffi_query, host: new_host)
      end

      # Query for a rule.
      #
      # @param name [String]
      # @param args [Array<Object>]
      # @return [Enumerator] of resulting bindings
      # @raise [Error] if the FFI call raises one.
      def query_rule(name, *args)
        query(Predicate.new(name, args: args))
      end

      # Register a Ruby class with Polar.
      #
      # @param cls [Class] the class to register.
      # @param name [String] the name to register the class as. Defaults to the name of the class.
      # @raise [DuplicateClassAliasError] if attempting to register a class
      # under a previously-registered name.
      # @raise [FFI::Error] if the FFI call returns an error.
      # @return [self] for chaining.
      def register_class(cls, name: nil)
        name = host.cache_class(cls, name: name || cls.name)
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

      # @return [FFI::Polar]
      attr_reader :ffi_polar
      attr_accessor :polar_roles_enabled

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
