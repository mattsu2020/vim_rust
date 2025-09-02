#ifndef RUST_MBYTE_H
#define RUST_MBYTE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int rust_utf_char2len(int c);
int rust_utf_char2bytes(int c, char *buf);
int rust_utf_ptr2len(const char *p);
int rust_utf_byte2len(int b);
int rust_utf_byte2len_zero(int b);
int rust_mb_charlen(const char *s);
int rust_utf_isupper(int c);
int rust_utf_islower(int c);
int rust_utf_toupper(int c);
int rust_utf_tolower(int c);

#ifdef __cplusplus
}
#endif

#endif // RUST_MBYTE_H
