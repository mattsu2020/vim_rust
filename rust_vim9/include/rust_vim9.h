#ifndef RUST_VIM9_H
#define RUST_VIM9_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct typval_S;
typedef struct typval_S typval_T;

bool vim9_exec_rs(const char *expr, typval_T *out);
long long vim9_eval_int(const char *expr);

#ifdef __cplusplus
}
#endif

#endif // RUST_VIM9_H
