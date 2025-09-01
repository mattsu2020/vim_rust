#ifndef RUST_EVAL_H
#define RUST_EVAL_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declaration of Vim's typval_T from structs.h
struct typval_S;
typedef struct typval_S typval_T;

bool eval_expr_rs(const char *expr, typval_T *out);

#ifdef __cplusplus
}
#endif

#endif // RUST_EVAL_H
