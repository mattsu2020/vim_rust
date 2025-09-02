/* vi:set ts=8 sts=4 sw=4 noet:
 * Simplified regular expression interface using the Rust backend.
 */

#ifndef _REGEXP_H
#define _REGEXP_H

#define NSUBEXP 10

#include "../rust_regex/include/rust_regex.h"

typedef RegProg regprog_T;
typedef RegMatch regmatch_T;

typedef struct {
    regprog_T *regprog;
    lpos_T startpos[NSUBEXP];
    lpos_T endpos[NSUBEXP];
    colnr_T rmm_matchcol;
    int rmm_ic;
    colnr_T rmm_maxcol;
} regmmatch_T;

typedef struct {
    short refcnt;
    char_u *matches[NSUBEXP];
} reg_extmatch_T;

int vim_rust_regex_match_wrapper(char_u *pat, char_u *text, int magic, long timeout_ms);

#define REGSUB_COPY         1
#define REGSUB_MAGIC        2
#define REGSUB_BACKSLASH    4

#endif // _REGEXP_H
