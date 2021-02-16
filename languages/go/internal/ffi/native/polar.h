/**
 * We use the convention of zero as an error term,
 * since we also use `null_ptr()` to indicate an error.
 * So for consistency, a zero term is an error in both cases.
 */
#define polar_POLAR_FAILURE 0

#define polar_POLAR_SUCCESS 1

typedef struct polar_Polar polar_Polar;

typedef struct polar_Query polar_Query;

const char *polar_get_error(void);

polar_Polar *polar_new(void);

int32_t polar_load(polar_Polar *polar_ptr, const char *src, const char *filename);

int32_t polar_clear_rules(polar_Polar *polar_ptr);

int32_t polar_register_constant(polar_Polar *polar_ptr, const char *name, const char *value);

polar_Query *polar_next_inline_query(polar_Polar *polar_ptr, uint32_t trace);

polar_Query *polar_new_query_from_term(polar_Polar *polar_ptr,
                                       const char *query_term,
                                       uint32_t trace);

polar_Query *polar_new_query(polar_Polar *polar_ptr, const char *query_str, uint32_t trace);

const char *polar_next_polar_message(polar_Polar *polar_ptr);

const char *polar_next_query_event(polar_Query *query_ptr);

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
int32_t polar_debug_command(polar_Query *query_ptr, const char *value);

int32_t polar_call_result(polar_Query *query_ptr, uint64_t call_id, const char *value);

int32_t polar_question_result(polar_Query *query_ptr, uint64_t call_id, int32_t result);

int32_t polar_application_error(polar_Query *query_ptr, char *message);

const char *polar_next_query_message(polar_Query *query_ptr);

const char *polar_query_source_info(polar_Query *query_ptr);

int32_t polar_bind(polar_Query *query_ptr, const char *name, const char *value);

uint64_t polar_get_external_id(polar_Polar *polar_ptr);

/**
 * Required to free strings properly
 */
int32_t string_free(char *s);

/**
 * Recovers the original boxed version of `polar` so that
 * it can be properly freed
 */
int32_t polar_free(polar_Polar *polar);

/**
 * Recovers the original boxed version of `query` so that
 * it can be properly freed
 */
int32_t query_free(polar_Query *query);
