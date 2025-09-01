#ifndef TERMINAL_FULL_H
#define TERMINAL_FULL_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Terminal Terminal;
typedef void (*terminal_line_cb)(const char *);

Terminal *terminal_full_new(int cols, int rows);
void terminal_full_free(Terminal *term);
void terminal_full_write(Terminal *term, const char *data);
void terminal_full_set_callback(Terminal *term, terminal_line_cb cb);
size_t terminal_full_get_scrollback(Terminal *term, size_t idx, char *buf, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* TERMINAL_FULL_H */
