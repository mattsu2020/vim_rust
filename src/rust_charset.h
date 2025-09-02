#ifndef RUST_CHARSET_H
#define RUST_CHARSET_H

#include <stdint.h>

unsigned nr2hex(unsigned c);
int hex2nr(int c);
int hexhex2nr(const unsigned char *p);

#endif // RUST_CHARSET_H
