# frozen_string_literal: true

require 'json'
require 'pp'
require 'set'
require 'digest/md5'

# Missing Ruby type.
module PolarBoolean; end
# Monkey-patch Ruby true type.
class TrueClass; include PolarBoolean; end
# Monkey-patch Ruby false type.
class FalseClass; include PolarBoolean; end

module Oso
  module Polar
    # Create and manage an instance of the Polar runtime.
    class Polar # rubocop:disable Metrics/ClassLength
      # @return [Host]
      attr_reader :host

      def initialize
        @ffi_polar = FFI::Polar.create
        @host = Host.new(ffi_polar)
        @loaded_names = {}
        @loaded_contents = {}

        # Register built-in classes.
        register_class PolarBoolean, name: 'Boolean'
        register_class Integer
        register_class Float
        register_class Array, name: 'List'
        register_class Hash, name: 'Dictionary'
        register_class String
      end

      # Replace the current Polar instance but retain all registered classes and constructors.
      def clear
        loaded_names.clear
        loaded_contents.clear
        @ffi_polar = FFI::Polar.create
      end

      # Enqueue a Polar policy file for loading into the KB.
      #
      # @param name [String]
      # @raise [PolarFileExtensionError] if provided filename has invalid extension.
      # @raise [PolarFileNotFoundError] if provided filename does not exist.
      def load_file(name) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
        raise PolarFileExtensionError, name unless File.extname(name) == '.polar'

        file_data = File.open(name, &:read)
        hash = Digest::MD5.hexdigest(file_data)

        if loaded_names.key?(name)
          raise PolarFileAlreadyLoadedError, name if loaded_names[name] == hash

          raise PolarFileContentsChangedError, name
        elsif loaded_contents.key?(hash)
          raise PolarFileNameChangedError, name, loaded_contents[hash]
        else
          load_str(file_data, filename: name)
          loaded_names[name] = hash
          loaded_contents[hash] = name
        end
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
      def load_str(str, filename: nil) # rubocop:disable Metrics/MethodLength
        raise NullByteInPolarFileError if str.chomp("\0").include?("\0")

        ffi_polar.load_str(str, filename: filename)
        loop do
          next_query = ffi_polar.next_inline_query
          break if next_query.nil?

          begin
            Query.new(next_query, host: host).results.next
          rescue StopIteration
            raise InlineQueryFailedError
          end
        end
      end

      # Query for a predicate, parsing it if necessary.
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
          ffi_query = ffi_polar.new_query_from_term(new_host.to_polar_term(query))
        else
          raise InvalidQueryTypeError
        end
        Query.new(ffi_query, host: new_host).results
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

      # Start a REPL session.
      #
      # @param files [Array<String>]
      # @raise [Error] if the FFI call raises one.
      def repl(files = []) # rubocop:disable Metrics/MethodLength, Metrics/CyclomaticComplexity, Metrics/PerceivedComplexity, Metrics/AbcSize
        files.map { |f| load_file(f) }

        loop do # rubocop:disable Metrics/BlockLength
          print 'query> '
          begin
            query = $stdin.readline.chomp.chomp(';')
          rescue EOFError
            return
          end

          begin
            ffi_query = ffi_polar.new_query_from_str(query)
          rescue ParseError => e
            puts "Parse error: #{e}"
            next
          end

          begin
            results = Query.new(ffi_query, host: host).results.to_a
          rescue PolarRuntimeError => e
            puts e
            next
          end

          if results.empty?
            pp false
          else
            results.each do |result|
              if result.empty?
                pp true
              else
                pp result
              end
            end
          end
        end
      end

      # Register a Ruby class with Polar.
      #
      # @param cls [Class]
      # @param name [String]
      # @param from_polar [Proc]
      # @raise [InvalidConstructorError] if provided an invalid 'from_polar' constructor.
      def register_class(cls, name: nil, from_polar: nil)
        from_polar = Proc.new if block_given?
        name = host.cache_class(cls, name: name, constructor: from_polar)
        register_constant(name, value: cls)
      end

      def register_constant(name, value:)
        ffi_polar.register_constant(name, value: host.to_polar_term(value))
      end

      private

      # @return [FFI::Polar]
      attr_reader :ffi_polar
      # @return [Hash<String, String>]
      attr_reader :loaded_names
      # @return [Hash<String, String>]
      attr_reader :loaded_contents
    end
  end
end
