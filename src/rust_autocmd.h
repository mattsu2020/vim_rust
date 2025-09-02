#ifndef RUST_AUTOCMD_H
#define RUST_AUTOCMD_H

#include <stdint.h>

int rs_autocmd_register(int event, const char *pattern, const char *cmd);
int rs_autocmd_execute(int event);

#endif // RUST_AUTOCMD_H
