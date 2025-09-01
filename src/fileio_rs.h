#ifndef FILEIO_RS_H
#define FILEIO_RS_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

long read_eintr(int fd, void *buf, size_t bufsize);
long write_eintr(int fd, const void *buf, size_t bufsize);

#ifdef __cplusplus
}
#endif

#endif // FILEIO_RS_H
