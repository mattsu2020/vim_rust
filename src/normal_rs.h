#ifndef NORMAL_RS_H
#define NORMAL_RS_H

#include "structs.h"

#ifdef __cplusplus
extern "C" {
#endif

void rs_normal_cmd(oparg_T *oap, int toplevel);
int rs_op_change(oparg_T *oap);

#ifdef __cplusplus
}
#endif

#endif // NORMAL_RS_H
