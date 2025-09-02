/* Minimal stubs for help to allow building with Rust-linked core. */
#include "vim.h"
#include "help_rs.h"

void ex_help(exarg_T *eap)
{
    // Minimal: just echo message to avoid breaking commands.
    emsg((char_u *)"help not built in this minimal configuration");
    (void)eap;
}

int find_help_tags(char_u *arg, int *num_matches, char_u ***matches, int keep_lang)
{
    return rs_find_help_tags((const char *)arg, num_matches, (char ***)matches, keep_lang);
}

void prepare_help_buffer(void) {}
void fix_help_buffer(void) {}
