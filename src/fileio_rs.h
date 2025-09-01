#ifndef FILEIO_RS_H
#define FILEIO_RS_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int rs_readfile(const char *fname);
int rs_writefile(const char *fname, const char *data, size_t len);

#ifdef __cplusplus
}
#endif

#endif // FILEIO_RS_H
