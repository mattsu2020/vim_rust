#ifndef RUST_TEXT_H
#define RUST_TEXT_H

#include <stdbool.h>

bool rs_add_text_prop(int id, const char *name);
char *rs_get_text_prop_name(int id);
char *rs_format_text(const char *input, bool uppercase, bool trim);
void rs_free_cstring(char *s);

#endif // RUST_TEXT_H
