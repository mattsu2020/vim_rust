#ifndef RUST_USERFUNC_H
#define RUST_USERFUNC_H

#include "vim.h"

int rust_userfunc_register(char_u *name, cfunc_T cb, cfunc_free_T cb_free, void *state);
int rust_userfunc_call(char_u *name, int argc, typval_T *argv, typval_T *rettv);
void rust_userfunc_clear(char_u *name);

#endif // RUST_USERFUNC_H
