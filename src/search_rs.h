#ifndef SEARCH_RS_H
#define SEARCH_RS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int cur;
    int cnt;
    int exact_match;
    int incomplete;
    int last_maxcount;
} searchstat_T;

// Update search statistics using the Rust implementation.  The function does
// not produce a return value; results are written directly into `stat`.
void rust_search_update_stat(const char *pat, const char *text,
                             searchstat_T *stat);

#ifdef __cplusplus
}
#endif

#endif // SEARCH_RS_H
