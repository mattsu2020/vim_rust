#ifndef INDENT_RS_H
#define INDENT_RS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int rs_cindent_level(const char *prev_line);
char *rs_complete_word(const char *prefix);

#ifdef __cplusplus
}
#endif

#endif // INDENT_RS_H
