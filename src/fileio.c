#include "vim.h"
#include "fileio_rs.h"
#include "path_rs.h"

/*
 * The original and complex C implementation for reading and writing files has
 * been replaced by a Rust version located in the rust_fileio crate.  This file
 * now only provides thin wrappers that forward to the Rust functions.
 */

int readfile(char_u *fname, char_u *sfname, linenr_T from,
             linenr_T lines_to_skip, linenr_T lines_to_read,
             exarg_T *eap, int flags)
{
    (void)sfname;
    (void)from;
    (void)lines_to_skip;
    (void)lines_to_read;
    (void)eap;
    (void)flags;
    char_u *norm = normalize_path(fname);
    int r = rs_readfile((const char *)norm);
    vim_free(norm);
    return r;
}

int writefile(char_u *fname, char_u *buf, size_t len, int flags)
{
    (void)flags;
    char_u *norm = normalize_path(fname);
    int r = rs_writefile((const char *)norm, (const char *)buf, len);
    vim_free(norm);
    return r;
}
