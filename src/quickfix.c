/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved    by Bram Moolenaar
 *
 * Do ":help uganda"  in Vim to read copying and usage conditions.
 * Do ":help credits" in Vim to see a list of people who contributed.
 */

/*
 * quickfix.c: wrappers calling into the Rust quickfix implementation
 */

#include "vim.h"
#include "../rust_quickfix/include/rust_quickfix.h"

int qf_add_entry(qf_list_T *qfl,
                 char_u *dir,
                 char_u *fname,
                 char_u *module,
                 int bufnum,
                 char_u *mesg,
                 long lnum,
                 long end_lnum,
                 int col,
                 int end_col,
                 int vis_col,
                 char_u *pattern,
                 int nr,
                 int type,
                 typval_T *user_data,
                 int valid)
{
    return rs_qf_add_entry((void *)qfl,
                           (const char *)dir,
                           (const char *)fname,
                           (const char *)module,
                           bufnum,
                           (const char *)mesg,
                           lnum,
                           end_lnum,
                           col,
                           end_col,
                           vis_col,
                           (const char *)pattern,
                           nr,
                           type,
                           user_data,
                           valid);
}

void qf_list(exarg_T *eap)
{
    rs_qf_list((void *)eap);
}
