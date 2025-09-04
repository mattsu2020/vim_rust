#ifndef OPS_RS_H
#define OPS_RS_H

/* structs.h は vim.h 側で読み込まれる */

#ifdef __cplusplus
extern "C" {
#endif

void rs_op_shift(oparg_T *oap, int curs_top, int amount);
int rs_op_delete(oparg_T *oap);
int rs_op_replace(oparg_T *oap, int c);
void rs_op_tilde(oparg_T *oap);
void rs_op_insert(oparg_T *oap, long count1);
int rs_op_change(oparg_T *oap);
void rs_op_addsub(oparg_T *oap, long Prenum1, int g_cmd);
void rs_op_colon(oparg_T *oap);
void rs_op_function(oparg_T *oap);
int rs_op_on_lines(int op);
int rs_skip_block_whitespace(struct block_def *bd, char_u *line, size_t line_len);

#ifdef __cplusplus
}
#endif

#endif // OPS_RS_H
