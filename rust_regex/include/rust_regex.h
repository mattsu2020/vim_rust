#ifndef RUST_REGEX_H
#define RUST_REGEX_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RegProg RegProg;

typedef struct RegMatch {
    const char *startp[10];
    const char *endp[10];
} RegMatch;

typedef struct SearchStat {
    int cur;
    int cnt;
    int exact_match;
    int incomplete;
    int last_maxcount;
} SearchStat;

RegProg* vim_regcomp(const char *pattern, int flags);
void vim_regfree(RegProg *prog);
int vim_regexec(RegProg *prog, const char *text, RegMatch *matchp);
char* vim_regsub(RegProg *prog, const char *text, const char *sub);
void rust_search_update_stat(const char *pat, const char *text, SearchStat *stat);
int rust_regex_match(const char *pat, const char *text, int magic, long timeout_ms);

#ifdef __cplusplus
}
#endif

#endif // RUST_REGEX_H
