#include "vim.h"
#include "drawscreen_rs.h"

// Forward screen update to Rust implementation.
void update_screen(int type_arg)
{
    rs_update_screen(type_arg);
}
