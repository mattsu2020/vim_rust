/* Minimal help stubs for Rust-integrated minimal build */
#include "vim.h"
#include "help_rs.h"

void ex_help(exarg_T *eap)
{
    (void)eap;
    emsg((char_u *)"help not available in this build");
}

int find_help_tags(char_u *arg, int *num_matches, char_u ***matches, int keep_lang)
{
    return rs_find_help_tags((const char *)arg, num_matches, (char ***)matches, keep_lang);
}

void prepare_help_buffer(void) {}
void fix_help_buffer(void) {}

