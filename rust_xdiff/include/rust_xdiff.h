#ifndef RUST_XDIFF_H
#define RUST_XDIFF_H

#ifdef __cplusplus
extern "C" {
#endif

char* vim_xdiff_diff(const char *a, const char *b);
void vim_xdiff_free(char *ptr);

#ifdef __cplusplus
}
#endif

#endif // RUST_XDIFF_H
