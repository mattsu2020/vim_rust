#ifndef RUST_RUBY_H
#define RUST_RUBY_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct typval_S;
typedef struct typval_S typval_T;
typedef unsigned char char_u;

void do_rubyeval(char_u *str, typval_T *rettv);

#ifdef __cplusplus
}
#endif

#endif // RUST_RUBY_H
