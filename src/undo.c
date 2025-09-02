#include "rust_undo.h"

UndoHistory *u_new_history(void)
{
    return rs_undo_history_new();
}

void u_free_history(UndoHistory *hist)
{
    rs_undo_history_free(hist);
}

int u_push(UndoHistory *hist, const char *text)
{
    return rs_undo_push(hist, text);
}

int u_pop(UndoHistory *hist, char *buf, size_t len)
{
    return rs_undo_pop(hist, buf, len);
}
