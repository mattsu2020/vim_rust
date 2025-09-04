#include "vim.h"

extern void rust_terminal_free_unused(void);
extern int rust_terminal_none_open(void *term);
extern void rust_terminal_clear_status_text(void *term);
extern char_u *rust_terminal_get_status_text(void *term);
extern void rust_terminal_set_status_text(void *term, const char_u *msg);
extern void rust_terminal_print(void *term, const char_u *msg);

void free_unused_terminals(void)
{
    rust_terminal_free_unused();
}

int term_none_open(void *term UNUSED)
{
    return rust_terminal_none_open(term);
}

void term_clear_status_text(void *term)
{
    rust_terminal_clear_status_text(term);
}

char_u *term_get_status_text(void *term)
{
    return rust_terminal_get_status_text(term);
}

void term_set_status_text(void *term, const char_u *msg)
{
    rust_terminal_set_status_text(term, msg);
}

void term_write(void *term, const char_u *msg)
{
    rust_terminal_print(term, msg);
}
