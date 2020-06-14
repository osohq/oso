# frozen_string_literal: true

require 'ffi'

module Osohq
  module Polar
    # TODO(gj): document
    module FFI
      extend ::FFI::Library

      LIB_PATH = "../../../../ext/osohq-polar/libpolar.#{::FFI::Platform::LIBSUFFIX}"

      ffi_lib [File.expand_path(File.join(__FILE__, LIB_PATH))]

      # int32_t polar_debug_command(polar_Polar *polar_ptr, polar_Query *query_ptr, const char *value);
      attach_function :polar_debug_command, %i[pointer pointer string], :int32

      # int32_t polar_external_call_result(polar_Polar *polar_ptr,
      #                                    polar_Query *query_ptr,
      #                                    uint64_t call_id,
      #                                    const char *value);
      attach_function :polar_external_call_result, %i[pointer pointer uint64 string], :int32

      # int32_t polar_external_question_result(polar_Polar *polar_ptr,
      #                                        polar_Query *query_ptr,
      #                                        uint64_t call_id,
      #                                        int32_t result);
      attach_function :polar_external_question_result, %i[pointer pointer uint64 string], :int32

      # int32_t polar_free(polar_Polar *polar);
      attach_function :polar_free, [:pointer], :int32

      # const char *polar_get_error(void);
      attach_function :polar_get_error, [], :string

      # uint64_t polar_get_external_id(polar_Polar *polar_ptr);
      attach_function :polar_get_external_id, [:pointer], :uint64

      # int32_t polar_load(polar_Polar *polar_ptr, polar_Load *load, polar_Query **query);
      attach_function :polar_load, %i[pointer pointer pointer], :int32

      # int32_t polar_load_str(polar_Polar *polar_ptr, const char *src);
      attach_function :polar_load_str, %i[pointer string], :int32

      # polar_Polar *polar_new(void);
      attach_function :polar_new, [], :pointer

      # polar_Load *polar_new_load(polar_Polar *polar_ptr, const char *src);
      attach_function :polar_new_load, %i[pointer string], :pointer

      # polar_Query *polar_new_query(polar_Polar *polar_ptr, const char *query_str);
      attach_function :polar_new_query, %i[pointer string], :pointer

      # polar_Query *polar_new_query_from_term(polar_Polar *polar_ptr, const char *query_term);
      attach_function :polar_new_query_from_term, %i[pointer pointer], :pointer

      # const char *polar_query(polar_Polar *polar_ptr, polar_Query *query_ptr);
      attach_function :polar_query, %i[pointer pointer], :string

      # polar_Query *polar_query_from_repl(polar_Polar *polar_ptr);
      attach_function :polar_query_from_repl, [:pointer], :pointer

      # int32_t load_free(polar_Load *load);
      attach_function :load_free, [:pointer], :int32

      # int32_t query_free(polar_Query *query);
      attach_function :query_free, [:pointer], :int32

      # int32_t string_free(char *s);
      attach_function :string_free, [:string], :int32

      # Check for an FFI error and convert it into a Ruby exception.
      #
      # @return [Osohq::Polar::Error] if there's an FFI error.
      # @raise [Osohq::Polar::FFIError] if there isn't one.
      def self.error
        error = polar_get_error
        raise Polar::FFIError if error.nil?

        kind, body = JSON.parse(error).first
        subkind, details = body.first
        case kind
        when 'Parse'
          return parse_error(kind: subkind, details: details)
        when 'Runtime'
          # TODO(gj): Runtime exception types.
          return ::Osohq::Polar::PolarRuntimeException.new(body: body)
        when 'Operational'
          return ::Osohq::Polar::InternalError.new if subkind == 'Unknown' # Rust panic.
        end
        # All errors should be mapped to Ruby exceptions.
        # Raise InternalError if we haven't mapped the error.
        Polar::InternalError.new(body)
      ensure
        string_free(error) unless error.nil?
      end

      # Map FFI parse errors into Ruby exceptions.
      #
      # @param kind [String]
      # @param details [Hash<String, Object>]
      # @return [Osohq::Polar::ParseError] the object converted into the expected format.
      def self.parse_error(kind:, details:)
        token = details['token']
        pos = details['pos']
        char = details['c']
        case kind
        when 'ExtraToken'
          Polar::ParseError::ExtraToken.new(token: token, pos: pos)
        when 'IntegerOverflow'
          Polar::ParseError::IntegerOverflow.new(token: token, pos: pos)
        when 'InvalidToken'
          Polar::ParseError::InvalidToken.new(pos: pos)
        when 'InvalidTokenCharacter'
          Polar::ParseError::InvalidTokenCharacter.new(token: token, char: char, pos: pos)
        when 'UnrecognizedEOF'
          Polar::ParseError::UnrecognizedEOF.new(pos: pos)
        when 'UnrecognizedToken'
          Polar::ParseError::UnrecognizedToken.new(token: token, pos: pos)
        else
          Polar::ParseError.new(kind: subkind, details: details)
        end
      end
    end
    private_constant :FFI
  end
end
