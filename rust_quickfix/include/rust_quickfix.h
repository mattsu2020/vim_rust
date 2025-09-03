#ifndef RUST_QUICKFIX_H
#define RUST_QUICKFIX_H

#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct typval_T typval_T;

int qf_add_entry(void *qfl,
                 const char *dir,
                 const char *fname,
                 const char *module,
                 int bufnum,
                 const char *mesg,
                 long lnum,
                 long end_lnum,
                 int col,
                 int end_col,
                 int vis_col,
                 const char *pattern,
                 int nr,
                 int typ,
                 typval_T *user_data,
                 int valid);

void qf_list(void *eap);

#ifdef __cplusplus
}
#endif

#endif // RUST_QUICKFIX_H
