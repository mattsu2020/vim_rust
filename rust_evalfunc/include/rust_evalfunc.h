#ifndef RUST_EVALFUNC_H
#define RUST_EVALFUNC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct typval_S;
typedef struct typval_S typval_T;

void f_hostname_rs(typval_T *argvars, typval_T *rettv);
void f_and_rs(typval_T *argvars, typval_T *rettv);

#ifdef __cplusplus
}
#endif

#endif // RUST_EVALFUNC_H
