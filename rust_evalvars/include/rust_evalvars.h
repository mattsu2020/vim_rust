#ifndef RUST_EVALVARS_H
#define RUST_EVALVARS_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

void rs_set_vim_var_nr(int32_t idx, int64_t val);
bool rs_get_vim_var_nr(int32_t idx, int64_t *out);
void rs_set_vim_var_str(int32_t idx, const char *val);
int64_t rs_eval_and(int64_t a, int64_t b);
int32_t rs_win_create(void);
int32_t rs_win_getid(int32_t winnr);

#ifdef __cplusplus
}
#endif

#endif // RUST_EVALVARS_H
