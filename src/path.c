#include "vim.h"
#include "path_rs.h"

/*
 * Thin wrappers around the Rust path utilities.
 */

char_u *normalize_path(char_u *path)
{
    return (char_u *)rs_normalize_path((const char *)path);
}

char_u *find_in_path(char_u *name, char_u *paths)
{
    return (char_u *)rs_find_in_path((const char *)name, (const char *)paths);
}
