#ifndef RUST_TUPLE_H
#define RUST_TUPLE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct rs_tuple rs_tuple;

rs_tuple *rs_tuple_new(size_t len);
bool rs_tuple_set(rs_tuple *tuple, size_t idx, int64_t value);
bool rs_tuple_get(const rs_tuple *tuple, size_t idx, int64_t *value);
size_t rs_tuple_len(const rs_tuple *tuple);
void rs_tuple_free(rs_tuple *tuple);

#endif // RUST_TUPLE_H
