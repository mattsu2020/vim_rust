#include "vim.h"

#if defined(FEAT_GUI_GTK) && defined(FEAT_GUI_RUST)
# include "gui_rust.h"

int gui_mch_init(void)
{
    rs_gui_run();
    return OK;
}
#endif
