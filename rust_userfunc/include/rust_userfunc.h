#ifndef RUST_USERFUNC_H
#define RUST_USERFUNC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct rust_funccall_S rust_funccall_T;

void rust_func_init(void);
void rust_func_deinit(void);

rust_funccall_T *rust_funccall_new(rust_funccall_T *previous, int32_t depth);
void rust_funccall_free(rust_funccall_T *fc);

int32_t rust_func_hashtab_set(const char *name, void *func);
void *rust_func_hashtab_get(const char *name);

#ifdef __cplusplus
}
#endif

#endif // RUST_USERFUNC_H
