#ifndef RUST_BUFFER_H
#define RUST_BUFFER_H

#include <stddef.h>

void *buf_alloc(size_t size);
void buf_freeall(void *buf, int flags);

#endif // RUST_BUFFER_H
