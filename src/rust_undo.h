#ifndef RUST_UNDO_H
#define RUST_UNDO_H

#include <stddef.h>

typedef struct UndoHistory UndoHistory;

UndoHistory *rs_undo_history_new(void);
void rs_undo_history_free(UndoHistory *hist);
int rs_undo_push(UndoHistory *hist, const char *text);
int rs_undo_pop(UndoHistory *hist, char *buf, size_t len);
int rs_undo_redo(UndoHistory *hist, char *buf, size_t len);

#endif // RUST_UNDO_H
