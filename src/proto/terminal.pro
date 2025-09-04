/* terminal stubs (minimal build) */
void free_unused_terminals(void);
int term_none_open(void *term);
void term_clear_status_text(void *term);
char_u *term_get_status_text(void *term);
/* vim: set ft=c : */
