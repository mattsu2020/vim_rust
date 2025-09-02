#ifndef RUST_OPTION_H
#define RUST_OPTION_H

#include <stdbool.h>
#include <stddef.h>

typedef struct rs_opt_t {
    const char *name;
    const char *default_value;
} rs_opt_t;

void rs_options_init(void);
bool rs_set_option(const char *name, const char *value);
char *rs_get_option(const char *name);
void rs_free_cstring(char *s);
const rs_opt_t *rs_get_option_defs(size_t *len);
bool rs_verify_option(const char *name);
bool rs_save_options(const char *path);
bool rs_load_options(const char *path);

#endif // RUST_OPTION_H
