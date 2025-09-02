#ifndef RUST_TERM_H
#define RUST_TERM_H

#include <stddef.h>

int rust_term_get_winsize(int *width, int *height);
int rust_term_color_count(void);
int rust_term_move_cursor(int x, int y);
int rust_term_clear_screen(void);
int rust_term_print(const char *s);

#endif // RUST_TERM_H
