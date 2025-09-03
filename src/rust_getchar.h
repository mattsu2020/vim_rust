#ifndef RUST_GETCHAR_H
#define RUST_GETCHAR_H

#include <stdint.h>
#include "rust_input.h"

int32_t rs_getchar(InputContext *ctx);
int32_t rs_getchar_avail(InputContext *ctx);
void rs_ungetchar(InputContext *ctx, uint32_t key);

#endif // RUST_GETCHAR_H
