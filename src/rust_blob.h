#ifndef RUST_BLOB_H
#define RUST_BLOB_H

#include <stddef.h>
#include <stdint.h>

typedef struct Blob Blob;

Blob *blob_alloc(void);
Blob *blob_ref(const Blob *b);
void blob_unref(const Blob *b);
size_t blob_len(const Blob *b);
uint8_t blob_get(const Blob *b, size_t idx);
void blob_set_append(const Blob *b, size_t idx, uint8_t byte);
int blob_equal(const Blob *b1, const Blob *b2);
Blob *blob_clone(const Blob *b);

#endif // RUST_BLOB_H
