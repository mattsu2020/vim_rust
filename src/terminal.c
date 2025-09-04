#include "vim.h"

// Stub implementations for terminal support in minimal builds.

void free_unused_terminals(void)
{
    // No terminals to free in this build.
}

int term_none_open(void *term UNUSED)
{
    return FAIL;
}

void term_clear_status_text(void *term UNUSED)
{
}

char_u *term_get_status_text(void *term UNUSED)
{
    return NULL;
}
