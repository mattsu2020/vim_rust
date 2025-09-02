#ifndef RUST_AUTOCMD_H
#define RUST_AUTOCMD_H

#include "vim.h"

void rust_autocmd_clear(void);
int rust_autocmd_add(int event, const char *pat, const char *cmd, int once, int nested);
int rust_autocmd_do(int event, const char *name);

#endif // RUST_AUTOCMD_H
