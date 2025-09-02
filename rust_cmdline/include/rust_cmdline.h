#ifndef RUST_CMDLINE_H
#define RUST_CMDLINE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int hist_char2type_rs(int c);
int history_add_rs(int hist_type, const char *line);
int history_len_rs(int hist_type);
const char *history_get_rs(int hist_type, int idx);

#ifdef __cplusplus
}
#endif

#endif // RUST_CMDLINE_H
