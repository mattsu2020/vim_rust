#ifndef RUST_BUFWRITE_H
#define RUST_BUFWRITE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int rs_buf_write(int fd, const void *buf, int len);

#ifdef __cplusplus
}
#endif

#endif // RUST_BUFWRITE_H
