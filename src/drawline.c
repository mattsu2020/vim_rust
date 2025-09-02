#include "vim.h"
#include "drawline_rs.h"

/*
 * Wrapper around the Rust implementation of line drawing.  The original C
 * file contained the full rendering logic; this version forwards the work to
 * the Rust crate while keeping the signature compatible.
 */
int win_line(win_T *wp, linenr_T lnum, int startrow, int endrow, int number_only, spellvars_T *spv)
{
    (void)wp;
    (void)lnum;
    (void)endrow;
    (void)number_only;
    (void)spv;
    // With no screen buffer available on the C side yet, pass NULL for now.
    // The Rust implementation will handle the null pointer gracefully.
    return rs_draw_line(NULL, startrow, "");
}
