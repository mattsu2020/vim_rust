#ifndef OPS_RS_H
#define OPS_RS_H

/* structs.h は vim.h 側で読み込まれる */

#ifdef __cplusplus
extern "C" {
#endif

int rs_op_change(oparg_T *oap);

#ifdef __cplusplus
}
#endif

#endif // OPS_RS_H
