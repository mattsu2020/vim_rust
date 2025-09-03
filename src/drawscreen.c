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

void update_debug_sign(buf_T *buf, linenr_T lnum) { (void)buf; (void)lnum; }
void updateWindow(win_T *wp) { (void)wp; }
int redraw_asap(int type) { set_must_redraw(type); return TRUE; }
void redraw_after_callback(int call_update_screen, int do_message)
{ (void)call_update_screen; (void)do_message; }
void redraw_later(int type) { set_must_redraw(type); }
void redraw_win_later(win_T *wp, int type) { (void)wp; set_must_redraw(type); }
void redraw_later_clear(void) { set_must_redraw(UPD_CLEAR); }
void redraw_all_later(int type) { set_must_redraw(type); }
void redraw_curbuf_later(int type) { set_must_redraw(type); }
void redraw_buf_later(buf_T *buf, int type) { (void)buf; set_must_redraw(type); }
void redraw_buf_line_later(buf_T *buf, linenr_T lnum) { (void)buf; (void)lnum; }
void redraw_buf_and_status_later(buf_T *buf, int type) { (void)buf; set_must_redraw(type); }
void redraw_statuslines(void) {}
void redrawWinline(win_T *wp, linenr_T lnum) { (void)wp; (void)lnum; }
void redraw_win_range_later(win_T *wp, linenr_T first, linenr_T last)
{ (void)wp; (void)first; (void)last; }

void redraw_cmd(int clear) { (void)clear; }
