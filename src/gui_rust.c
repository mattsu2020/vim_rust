#include "vim.h"
#include "gui_rust.h"

/*
 * Minimal C shell delegating GUI handling to the Rust implementation.
 */
void gui_start(char_u *arg UNUSED)
{
    rs_gui_run();
}
