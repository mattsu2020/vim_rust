#ifndef RUST_SEARCH_H
#define RUST_SEARCH_H

#include "vim.h"

typedef struct searchit_arg_T searchit_arg_T;

typedef struct {
    int cur;
    int cnt;
    int exact_match;
    int incomplete;
    int last_maxcount;
} searchstat_T;

int rust_searchit(win_T *win, buf_T *buf, pos_T *pos, pos_T *end_pos,
                  int dir, char_u *pat, size_t patlen, long count,
                  int options, int pat_use, searchit_arg_T *extra_arg);

void rust_find_pattern_in_path(char_u *ptr, int dir, int len, int whole,
                               int skip_comments, int type, long count,
                               int action, linenr_T start_lnum,
                               linenr_T end_lnum, int forceit, int silent);

int rust_search_regcomp(char_u *pat, size_t patlen, char_u **used_pat,
                        int pat_save, int pat_use, int options,
                        void *regmatch);
char_u *rust_get_search_pat(size_t *len);
void rust_save_re_pat(int idx, char_u *pat, size_t patlen, int magic);
void rust_save_search_patterns(void);
void rust_restore_search_patterns(void);
void rust_free_search_patterns(void);
void rust_save_last_search_pattern(void);
void rust_restore_last_search_pattern(void);
char_u *rust_last_search_pattern(void);
size_t rust_last_search_pattern_len(void);

int rust_search_update_stat(const char *pat, const char *text, searchstat_T *stat);

#endif // RUST_SEARCH_H
