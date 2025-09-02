#ifndef RUST_HASHTAB_H
#define RUST_HASHTAB_H

#include <stddef.h>

void* rust_hashtab_new(void);
void rust_hashtab_free(void* tab);
int rust_hashtab_set(void* tab, const char* key, void* value);
void* rust_hashtab_get(void* tab, const char* key);
int rust_hashtab_remove(void* tab, const char* key);

#endif
