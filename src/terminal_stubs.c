#include "vim.h"

int term_none_open(void *term UNUSED)
{
    return 0;
}

void term_clear_status_text(void *term UNUSED)
{
}

char_u *term_get_status_text(void *term UNUSED)
{
    static char_u empty[] = "";
    return empty;
}

