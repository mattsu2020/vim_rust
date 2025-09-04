#ifndef RUST_DIFF_H
#define RUST_DIFF_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char *ptr;
    long size;
} mmfile_t;

typedef struct {
    char *ptr;
    long size;
} mmbuffer_t;

typedef struct {
    uint64_t flags;
    char **anchors;
    size_t anchors_nr;
} xpparam_t;

typedef int (*out_hunk_fn)(void *, long, long, long, long, const char *, long);
typedef int (*out_line_fn)(void *, mmbuffer_t *, int);

typedef struct {
    void *priv_;
    out_hunk_fn out_hunk;
    out_line_fn out_line;
} xdemitcb_t;

typedef struct {
    long ctxlen;
    long interhunkctxlen;
    uint64_t flags;
    long (*find_func)(const char *, long, char *, long, void *);
    void *find_func_priv;
    int (*hunk_func)(long, long, long, long, void *);
} xdemitconf_t;

/* flag values compatible with the historical xdiff API */
#define XDF_NEED_MINIMAL           (1 << 0)
#define XDF_IGNORE_WHITESPACE      (1 << 1)
#define XDF_IGNORE_WHITESPACE_CHANGE (1 << 2)
#define XDF_IGNORE_WHITESPACE_AT_EOL (1 << 3)
#define XDF_IGNORE_CR_AT_EOL       (1 << 4)
#define XDF_WHITESPACE_FLAGS (XDF_IGNORE_WHITESPACE | \
                              XDF_IGNORE_WHITESPACE_CHANGE | \
                              XDF_IGNORE_WHITESPACE_AT_EOL | \
                              XDF_IGNORE_CR_AT_EOL)
#define XDF_IGNORE_BLANK_LINES     (1 << 7)
#define XDF_PATIENCE_DIFF          (1 << 14)
#define XDF_HISTOGRAM_DIFF         (1 << 15)
#define XDF_DIFF_ALGORITHM_MASK    (XDF_PATIENCE_DIFF | XDF_HISTOGRAM_DIFF)
#define XDF_DIFF_ALG(x)            ((x) & XDF_DIFF_ALGORITHM_MASK)
#define XDF_INDENT_HEURISTIC       (1 << 23)

int xdl_diff(const mmfile_t *mf1, const mmfile_t *mf2,
             const xpparam_t *xpp, const xdemitconf_t *xecfg,
             xdemitcb_t *ecb);
int xdiff_out_unified(void *priv_, mmbuffer_t *mb, int nbuf);
int xdiff_out_indices(long start_a, long count_a,
                      long start_b, long count_b, void *priv_);

size_t linematch_nbuffers(const mmfile_t **diff_blk, const int *diff_len,
                          size_t ndiffs, int **decisions, int iwhite);

void rs_diff_update_screen(void);

#ifdef __cplusplus
}
#endif

#endif // RUST_DIFF_H
