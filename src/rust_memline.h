#ifndef RUST_MEMLINE_H
#define RUST_MEMLINE_H

#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

void *rs_ml_buffer_new(void);
void rs_ml_buffer_free(void *buf);
bool rs_ml_append(void *buf, size_t lnum, const char *line);
bool rs_ml_delete(void *buf, size_t lnum);
bool rs_ml_replace(void *buf, size_t lnum, const char *line);
unsigned char *rs_ml_get_line(void *buf, size_t lnum, int for_change, size_t *out_len);
size_t rs_ml_line_count(void *buf);

#ifdef __cplusplus
}
#endif

#endif // RUST_MEMLINE_H
