/**
 * We use the convention of zero as an error term,
 * since we also use `null_ptr()` to indicate an error.
 * So for consistency, a zero term is an error in both cases.
 */
#define polar_POLAR_FAILURE 0

#define polar_POLAR_SUCCESS 1

typedef struct polar_Polar polar_Polar;

typedef struct polar_Query polar_Query;

/**
 * Wrapper struct to help us return errors
 */
typedef struct polar_CResult_c_void {
  void *result;
  const char *error;
} polar_CResult_c_void;

/**
 * Wrapper struct to help us return errors
 */
typedef struct polar_CResult_Query {
  struct polar_Query *result;
  const char *error;
} polar_CResult_Query;

/**
 * Wrapper struct to help us return errors
 */
typedef struct polar_CResult_c_char {
  char *result;
  const char *error;
} polar_CResult_c_char;

struct polar_Polar *polar_new(void);

struct polar_CResult_c_void *polar_load(struct polar_Polar *polar_ptr, const char *sources);

struct polar_CResult_c_void *polar_clear_rules(struct polar_Polar *polar_ptr);

struct polar_CResult_c_void *polar_register_constant(struct polar_Polar *polar_ptr,
                                                     const char *name,
                                                     const char *value);

struct polar_CResult_c_void *polar_register_mro(struct polar_Polar *polar_ptr,
                                                const char *name,
                                                const char *mro);

struct polar_Query *polar_next_inline_query(struct polar_Polar *polar_ptr, uint32_t trace);

struct polar_CResult_Query *polar_new_query_from_term(struct polar_Polar *polar_ptr,
                                                      const char *query_term,
                                                      uint32_t trace);

struct polar_CResult_Query *polar_new_query(struct polar_Polar *polar_ptr,
                                            const char *query_str,
                                            uint32_t trace);

struct polar_CResult_c_char *polar_next_polar_message(struct polar_Polar *polar_ptr);

struct polar_CResult_c_char *polar_next_query_event(struct polar_Query *query_ptr);

/**
 * Execute one debugger command for the given query.
 *
 * ## Returns
 * - `0` on error.
 * - `1` on success.
 *
 * ## Errors
 * - Provided value is NULL.
 * - Provided value contains malformed JSON.
 * - Provided value cannot be parsed to a Term wrapping a Value::String.
 * - Query.debug_command returns an error.
 * - Anything panics during the parsing/execution of the provided command.
 */
struct polar_CResult_c_void *polar_debug_command(struct polar_Query *query_ptr, const char *value);

struct polar_CResult_c_void *polar_call_result(struct polar_Query *query_ptr,
                                               uint64_t call_id,
                                               const char *term);

struct polar_CResult_c_void *polar_question_result(struct polar_Query *query_ptr,
                                                   uint64_t call_id,
                                                   int32_t result);

struct polar_CResult_c_void *polar_application_error(struct polar_Query *query_ptr, char *message);

struct polar_CResult_c_char *polar_next_query_message(struct polar_Query *query_ptr);

struct polar_CResult_c_char *polar_query_source_info(struct polar_Query *query_ptr);

struct polar_CResult_c_void *polar_bind(struct polar_Query *query_ptr,
                                        const char *name,
                                        const char *value);

uint64_t polar_get_external_id(struct polar_Polar *polar_ptr);

/**
 * Required to free strings properly
 */
int32_t string_free(char *s);

/**
 * Recovers the original boxed version of `polar` so that
 * it can be properly freed
 */
int32_t polar_free(struct polar_Polar *polar);

/**
 * Recovers the original boxed version of `query` so that
 * it can be properly freed
 */
int32_t query_free(struct polar_Query *query);

/**
 * Recovers the original boxed version of `result` so that
 * it can be properly freed
 */
int32_t result_free(struct polar_CResult_c_void *result);

struct polar_CResult_c_char *polar_build_data_filter(struct polar_Polar *polar_ptr,
                                                     const char *types,
                                                     const char *results,
                                                     const char *variable,
                                                     const char *class_tag);

struct polar_CResult_c_char *polar_build_filter_plan(struct polar_Polar *polar_ptr,
                                                     const char *types,
                                                     const char *results,
                                                     const char *variable,
                                                     const char *class_tag);
