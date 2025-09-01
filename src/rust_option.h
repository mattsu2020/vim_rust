#ifndef RUST_OPTION_H
#define RUST_OPTION_H

#include <stdbool.h>

void rs_options_init(void);
bool rs_set_option(const char *name, const char *value);
char *rs_get_option(const char *name);
void rs_free_cstring(char *s);

#endif // RUST_OPTION_H
