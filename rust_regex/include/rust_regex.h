#ifndef RUST_REGEX_H
#define RUST_REGEX_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RegProg RegProg;
struct regmatch_T;
struct regmmatch_T;
struct win_T;
struct buf_T;

RegProg* vim_regcomp(const char *pattern, int flags);
void vim_regfree(RegProg *prog);
int vim_regexec(struct regmatch_T *rmp, const char *line, int col);
int vim_regexec_nl(struct regmatch_T *rmp, const char *line, int col);
long vim_regexec_multi(struct regmmatch_T *rmp, struct win_T *win,
                       struct buf_T *buf, long lnum, int col,
                       int *timed_out);
char* vim_regsub(RegProg *prog, const char *text, const char *sub);

#ifdef __cplusplus
}
#endif

#endif // RUST_REGEX_H
