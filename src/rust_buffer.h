#ifndef RUST_BUFFER_H
#define RUST_BUFFER_H

#include <stddef.h>
#include "structs.h"

void *buf_alloc(size_t size);
void buf_freeall(buf_T *buf, int flags);

#endif // RUST_BUFFER_H
