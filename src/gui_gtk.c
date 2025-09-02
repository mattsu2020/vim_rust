#include "vim.h"

#ifdef FEAT_GUI_GTK

extern void rs_gui_run(void);

int gui_mch_init(void)
{
    rs_gui_run();
    return OK;
}

#endif
