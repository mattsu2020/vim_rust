#ifndef RUST_VIM9CMDS_H
#define RUST_VIM9CMDS_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct typval_S;
typedef struct typval_S typval_T;

bool vim9_exec_rs(const char *expr, typval_T *out);
long long vim9_eval_int(const char *expr);
bool vim9_eval_bool(const char *expr);
void vim9_declare_error_rs(const char *name);

#ifdef __cplusplus
}
#endif

#endif // RUST_VIM9CMDS_H
