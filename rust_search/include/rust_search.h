#ifndef RUST_SEARCH_H
#define RUST_SEARCH_H

#include "vim.h"

typedef struct searchit_arg_T searchit_arg_T;

int rust_searchit(win_T *win, buf_T *buf, pos_T *pos, pos_T *end_pos,
                  int dir, char_u *pat, size_t patlen, long count,
                  int options, int pat_use, searchit_arg_T *extra_arg);

void rust_find_pattern_in_path(char_u *ptr, int dir, int len, int whole,
                               int skip_comments, int type, long count,
                               int action, linenr_T start_lnum,
                               linenr_T end_lnum, int forceit, int silent);

#endif // RUST_SEARCH_H
