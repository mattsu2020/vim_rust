#ifndef RUST_DIFF_H
#define RUST_DIFF_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    RUST_DIFF_EXTERNAL = 0,
    RUST_DIFF_XDIFF = 1,
} DiffMode;

char *rs_diff_files(const char *file1, const char *file2, DiffMode mode);
void rs_diff_free(char *ptr);
void rs_diff_update_screen(void);

#ifdef __cplusplus
}
#endif

#endif // RUST_DIFF_H
