#ifndef NORMAL_RS_H
#define NORMAL_RS_H

/* structs.h は vim.h 経由で読み込まれる前提 */

#ifdef __cplusplus
extern "C" {
#endif

void rs_normal_cmd(oparg_T *oap, int toplevel);
void rs_del_from_showcmd(int len);
int rs_check_text_or_curbuf_locked(oparg_T *oap);

#ifdef __cplusplus
}
#endif

#endif // NORMAL_RS_H
