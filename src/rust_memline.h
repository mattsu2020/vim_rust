#ifndef RUST_MEMLINE_H
#define RUST_MEMLINE_H

#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MemBuffer MemBuffer;

MemBuffer *ml_buffer_new(void);
void ml_buffer_free(MemBuffer *buf);
bool ml_append(MemBuffer *buf, size_t lnum, const char *line);
bool ml_delete(MemBuffer *buf, size_t lnum);
bool ml_replace(MemBuffer *buf, size_t lnum, const char *line);
unsigned char *ml_get_line(MemBuffer *buf, size_t lnum, int for_change, size_t *out_len);
size_t ml_line_count(MemBuffer *buf);

#ifdef __cplusplus
}
#endif

#endif // RUST_MEMLINE_H
