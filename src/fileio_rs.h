#ifndef FILEIO_RS_H
#define FILEIO_RS_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int rs_readfile(const char *fname, const char *sfname, long from, long lines_to_skip,
                long lines_to_read, void *eap, int flags);
int rs_writefile(const char *fname, const char *data, size_t len, int flags);

#ifdef __cplusplus
}
#endif

#endif // FILEIO_RS_H
