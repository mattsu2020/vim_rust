#ifndef RUST_ARABIC_H
#define RUST_ARABIC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int arabic_maycombine(int two);
int arabic_combine(int one, int two);
int arabic_shape(int c, int *ccp, int *c1p, int prev_c, int prev_c1, int next_c);

#ifdef __cplusplus
}
#endif

#endif // RUST_ARABIC_H
