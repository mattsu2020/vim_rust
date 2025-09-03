// Minimal compatibility for help.c functions used by other modules.
#include "vim.h"

char_u *check_help_lang(char_u *arg)
{
    return arg;
}

int help_heuristic(char_u *matched_string, int offset, int wrong_case)
{
    (void)matched_string; (void)offset; (void)wrong_case; return 0;
}

int find_help_tags(char_u *arg, int *num_matches, char_u ***matches, int keep_lang)
{
    (void)arg; (void)keep_lang;
    if (num_matches) *num_matches = 0;
    if (matches) *matches = NULL;
    return OK;
}

void cleanup_help_tags(int num_file, char_u **file)
{
    (void)num_file; (void)file;
}

