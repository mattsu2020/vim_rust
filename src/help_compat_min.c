// Minimal helper to provide missing help API pieces when using Rust help.
#include "vim.h"

char_u *check_help_lang(char_u *arg)
{
    return arg;
}

int help_heuristic(char_u *matched_string, int offset, int wrong_case)
{
    (void)matched_string; (void)offset; (void)wrong_case; return 0;
}

void cleanup_help_tags(int num_file, char_u **file)
{
    (void)num_file; (void)file;
}
