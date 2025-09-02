#ifndef RUST_TERM_H
#define RUST_TERM_H

#include <stddef.h>

typedef struct rust_term Terminal;

Terminal *rust_term_new(void);
void rust_term_free(Terminal *term);

int rust_term_out_char(Terminal *term, int c);
int rust_term_out_flush(Terminal *term);

int rust_term_move_cursor(Terminal *term, int x, int y);
int rust_term_clear_screen(Terminal *term);
int rust_term_print(Terminal *term, const char *s);

int rust_term_get_winsize(int *width, int *height);
int rust_term_color_count(void);

#endif // RUST_TERM_H
