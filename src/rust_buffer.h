#ifndef RUST_BUFFER_H
#define RUST_BUFFER_H

#include <stddef.h>
/* structs.h は vim.h 側で読み込まれる */

void *buf_alloc(size_t size);
void buf_free(void *buf);
void buf_freeall(void *buf, int flags);

#endif // RUST_BUFFER_H
