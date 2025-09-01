#ifndef RUST_MEM_H
#define RUST_MEM_H

#include <stddef.h>
#include <string.h>

#define rs_memchr(s, c, n) ((char *)memchr((s), (c), (n)))

#endif // RUST_MEM_H
