#ifndef RUST_MBYTE_H
#define RUST_MBYTE_H

#include <stdint.h>

int utf_char2len(uint32_t c);
int utf_char2bytes(uint32_t c, uint8_t *buf);
int utf_ptr2len(const uint8_t *p);
int utf_ptr2char(const uint8_t *p, uint32_t *c);

#endif // RUST_MBYTE_H
