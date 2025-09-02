#include "vim.h"
#include "drawscreen_rs.h"

// Forward screen update to Rust implementation.
void update_screen(int type_arg)
{
    rs_update_screen(type_arg);
}

// Minimal no-op stubs to satisfy references from other modules.
int statusline_row(win_T *wp)
{
    (void)wp;
    return 0;
}

void after_updating_screen(int may_resize_shell)
{
    (void)may_resize_shell;
}

void update_curbuf(int type)
{
    (void)type;
}

void win_redr_status(win_T *wp, int ignore_pum)
{
    (void)wp;
    (void)ignore_pum;
}

void win_redr_ruler(win_T *wp, int always, int ignore_pum)
{
    (void)wp; (void)always; (void)ignore_pum;
}

void status_redraw_all(void)
{
}

void status_redraw_curbuf(void)
{
}

void win_redraw_last_status(frame_T *frp)
{
    (void)frp;
}

void update_topline_cursor(void)
{
}
