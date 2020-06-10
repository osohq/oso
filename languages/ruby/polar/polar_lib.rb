require 'ffi'

module PolarLib
    extend FFI::Library
    ffi_lib [Dir.pwd+'/../../../target/debug/libpolar.dylib']

    # int32_t polar_debug_command(polar_Polar *polar_ptr, polar_Query *query_ptr, const char *value);
    attach_function :polar_debug_command, [:pointer, :pointer, :string], :int32

    # int32_t polar_external_call_result(polar_Polar *polar_ptr,
    #                                    polar_Query *query_ptr,
    #                                    uint64_t call_id,
    #                                    const char *value);
    attach_function :polar_external_call_result, [:pointer, :pointer, :uint64, :string], :int32

    # int32_t polar_external_question_result(polar_Polar *polar_ptr,
    #                                        polar_Query *query_ptr,
    #                                        uint64_t call_id,
    #                                        int32_t result);
    attach_function :polar_external_question_result, [:pointer, :pointer, :uint64, :string], :int32

    # int32_t polar_free(polar_Polar *polar);
    attach_function :polar_free, [:pointer], :int32

    # const char *polar_get_error(void);
    attach_function :polar_get_error, [ ], :string

    # uint64_t polar_get_external_id(polar_Polar *polar_ptr);
    attach_function :polar_get_external_id, [:pointer], :uint64

    # int32_t polar_load(polar_Polar *polar_ptr, polar_Load *load, polar_Query **query);
    attach_function :polar_load, [:pointer, :pointer, :pointer], :int32

    # int32_t polar_load_str(polar_Polar *polar_ptr, const char *src);
    attach_function :polar_load_str, [:pointer, :string], :int

    # polar_Polar *polar_new(void);
    attach_function :polar_new, [ ], :pointer

    # polar_Load *polar_new_load(polar_Polar *polar_ptr, const char *src);
    attach_function :polar_new_load, [:pointer, :string], :pointer

    # polar_Query *polar_new_query(polar_Polar *polar_ptr, const char *query_str);
    attach_function :polar_new_query, [:pointer, :string], :pointer

    # polar_Query *polar_new_query_from_term(polar_Polar *polar_ptr, const char *query_term);
    attach_function :polar_new_query_from_term, [:pointer, :pointer], :pointer

    # const char *polar_query(polar_Polar *polar_ptr, polar_Query *query_ptr);
    attach_function :polar_query, [:pointer, :pointer], :string

    # polar_Query *polar_query_from_repl(polar_Polar *polar_ptr);
    attach_function :polar_query_from_repl, [:pointer], :pointer

    # int32_t query_free(polar_Query *query);
    attach_function :query_free, [:pointer], :int32

    # int32_t string_free(char *s);
    attach_function :string_free, [:string], :int32
end