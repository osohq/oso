require 'ffi'

module PolarLib
    extend FFI::Library
    ffi_lib [Dir.pwd+'/../../../target/debug/libpolar.dylib']
    attach_function :polar_new, [ ], :pointer
    attach_function :polar_debug_command, [:pointer, :pointer, :string], :int
    attach_function :polar_load_str, [:pointer, :string], :int
    attach_function :polar_new_query, [:pointer, :string], :pointer

# int32_t polar_external_call_result(polar_Polar *polar_ptr,
#                                    polar_Query *query_ptr,
#                                    uint64_t call_id,
#                                    const char *value);
#
# int32_t polar_external_question_result(polar_Polar *polar_ptr,
#                                        polar_Query *query_ptr,
#                                        uint64_t call_id,
#                                        int32_t result);
#
# int32_t polar_free(polar_Polar *polar);
#
# const char *polar_get_error(void);
#
# uint64_t polar_get_external_id(polar_Polar *polar_ptr);
#
# int32_t polar_load(polar_Polar *polar_ptr, polar_Load *load, polar_Query **query);
#
#
# polar_Load *polar_new_load(polar_Polar *polar_ptr, const char *src);
#
#
# polar_Query *polar_new_query_from_term(polar_Polar *polar_ptr, const char *query_term);
#
# const char *polar_query(polar_Polar *polar_ptr, polar_Query *query_ptr);
#
# polar_Query *polar_query_from_repl(polar_Polar *polar_ptr);
#
# int32_t query_free(polar_Query *query);
#
# int32_t string_free(char *s);
end