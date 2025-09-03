#include "vim.h"
#include "fileio_rs.h"

/*
 * Thin wrappers around the Rust file I/O utilities.
 */

int readfile(
    char_u *fname,
    char_u *sfname,
    linenr_T from,
    linenr_T lines_to_skip,
    linenr_T lines_to_read,
    exarg_T *eap,
    int flags)
{
    return rs_readfile(
        (const char *)fname,
        (const char *)sfname,
        (long)from,
        (long)lines_to_skip,
        (long)lines_to_read,
        (void *)eap,
        flags);
}

int writefile(
    char_u *fname,
    char_u *buf,
    size_t len,
    int flags)
{
    return rs_writefile((const char *)fname, (const char *)buf, len, flags);
}
