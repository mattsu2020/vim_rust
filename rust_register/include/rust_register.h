#ifndef RUST_REGISTER_H
#define RUST_REGISTER_H

#include <stdbool.h>
#include <stddef.h>

int rs_register_set(char reg, const char *value);
char *rs_register_get(char reg);
void rs_register_free(char *s);

#endif // RUST_REGISTER_H
