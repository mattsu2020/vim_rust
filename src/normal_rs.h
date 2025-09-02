#ifndef NORMAL_RS_H
#define NORMAL_RS_H

/* structs.h は vim.h 経由で読み込まれる前提 */

#ifdef __cplusplus
extern "C" {
#endif

void rs_normal_cmd(oparg_T *oap, int toplevel);

#ifdef __cplusplus
}
#endif

#endif // NORMAL_RS_H
