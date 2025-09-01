#ifndef TERMINAL_RS_H
#define TERMINAL_RS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Terminal Terminal;

Terminal* terminal_new(int width, int height);
void terminal_free(Terminal* term);
int terminal_write_input(Terminal* term, const char *data, size_t len);
int terminal_read_output(Terminal* term);
int terminal_scrollback_len(Terminal* term);
const char* terminal_scrollback_line(Terminal* term, int idx);

#ifdef __cplusplus
}
#endif

#endif // TERMINAL_RS_H
