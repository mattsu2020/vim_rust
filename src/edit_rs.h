#ifndef EDIT_RS_H
#define EDIT_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int rs_ins_char(int c);
int rs_del_char(int fixpos);
int rs_backspace_char(void);

#ifdef __cplusplus
}
#endif

#endif // EDIT_RS_H
