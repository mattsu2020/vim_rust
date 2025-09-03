#ifndef RUST_REGEX_ENGINE_H
#define RUST_REGEX_ENGINE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RegProg RegProg;

// Lightweight copies of the structures used by Vim's C code.  Keeping the
// definitions here makes the FFI interface self-contained and allows external
// consumers to allocate and read these structs without including Vim headers.
typedef struct {
    long lnum;
    int col;
} Lpos;

typedef struct {
    RegProg *regprog;
    const char *startp[10];
    const char *endp[10];
    int rm_matchcol;
    int rm_ic;
} RegMatch;

typedef struct {
    RegProg *regprog;
    Lpos startpos[10];
    Lpos endpos[10];
    int rmm_matchcol;
    int rmm_ic;
    int rmm_maxcol;
} RegMMMatch;

RegProg* vim_regcomp(const char *pattern, int flags);
void vim_regfree(RegProg *prog);
int vim_regexec(RegMatch *rmp, const char *line, int col);
int vim_regexec_nl(RegMatch *rmp, const char *line, int col);
long vim_regexec_multi(RegMMMatch *rmp, void *win,
                       void *buf, long lnum, int col,
                       int *timed_out);
char* vim_regsub(RegProg *prog, const char *text, const char *sub);

#ifdef __cplusplus
}
#endif

#endif // RUST_REGEX_ENGINE_H
