#ifndef RUST_EVAL_H
#define RUST_EVAL_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declaration of Vim's typval_T from structs.h
struct typval_S;
typedef struct typval_S typval_T;

bool eval_expr_rs(const char *expr, typval_T *out);
// Minimal replacement for Vim's eval_to_string()
// Returns a newly allocated NUL-terminated string (caller frees).
unsigned char *eval_to_string(const unsigned char *expr,
                              int use_sandbox,
                              int remove_quotes);
bool eval_to_bool_rs(const char *expr, bool *error);
bool eval_variable_rs(const char *name, typval_T *out);
bool set_variable_rs(const char *name, const typval_T *val);
bool call_function_rs(const char *name, const typval_T *args, size_t argc,
                      typval_T *out);
bool eval_script_rs(const char *script, typval_T *out);

#ifdef __cplusplus
}
#endif

#endif // RUST_EVAL_H
