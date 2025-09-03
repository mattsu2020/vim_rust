#ifndef RUST_CORE_H
#define RUST_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct typval_S;
typedef struct typval_S typval_T;

void tv_number(int64_t n, typval_T *out);
void tv_string(const char *s, typval_T *out);
void tv_free(typval_T *tv);

void *vim_alloc_rs(size_t size);
void *alloc_clear_rs(size_t size);
void vim_free_rs(void *ptr);
void *mem_realloc_rs(void *ptr, size_t size);

#ifdef __cplusplus
}
#endif

#endif // RUST_CORE_H
