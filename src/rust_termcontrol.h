#ifndef RUST_TERMCONTROL_H
#define RUST_TERMCONTROL_H

#include <stddef.h>

int rust_termcontrol_color_count(void);
int rust_termcontrol_move_cursor(int x, int y);
int rust_termcontrol_clear_screen(void);
int rust_termcontrol_print(const char *s);

#endif // RUST_TERMCONTROL_H
