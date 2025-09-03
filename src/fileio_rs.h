#ifndef FILEIO_RS_H
#define FILEIO_RS_H

#include <stddef.h>

#ifdef __cplusplus
extern "C"
{
#endif

    int readfile(const char *fname,
                 const char *sfname,
                 long from,
                 long lines_to_skip,
                 long lines_to_read,
                 void *eap,
                 int flags);
    int writefile(const char *fname, const char *data, size_t len, int flags);
    char *rs_findfile(const char *name, const char *path);
    char *rs_read_viminfo(const char *path);
    int rs_write_viminfo(const char *path, const char *data);

#ifdef __cplusplus
}
#endif

#endif // FILEIO_RS_H
