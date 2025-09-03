#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

// change.c API (now provided by rust_change)

// Equivalent of Vim's change_warning(). Returns 1 when a warning should be
// displayed and 0 otherwise. Only warns once while the buffer is readonly and
// not yet changed.
int change_warning(int _col);

// Mark the buffer as changed (mirrors Vim's changed()).
void changed(void);

// Internal variant used in a few places.
void changed_internal(void);

// Listener related helpers (no-ops in minimal implementations)
void f_listener_add(typval_T *argvars, typval_T *rettv);
void f_listener_flush(typval_T *argvars, typval_T *rettv);
void f_listener_remove(typval_T *argvars, typval_T *rettv);
void may_invoke_listeners(buf_T *buf, linenr_T lnum, linenr_T lnume, int added);
void invoke_listeners(buf_T *buf);
void remove_listeners(buf_T *buf);

// Change notifications
void changed_bytes(linenr_T lnum, colnr_T col);
void inserted_bytes(linenr_T lnum, colnr_T col, int added);
void appended_lines(linenr_T lnum, long count);
void appended_lines_mark(linenr_T lnum, long count);
void deleted_lines(linenr_T lnum, long count);
void deleted_lines_mark(linenr_T lnum, long count);
void changed_lines_buf(buf_T *buf, linenr_T lnum, linenr_T lnume, long xtra);
void changed_lines(linenr_T lnum, colnr_T col, linenr_T lnume, long xtra);
void unchanged(buf_T *buf, int ff, int always_inc_changedtick);
void save_file_ff(buf_T *buf);
int  file_ff_differs(buf_T *buf, int ignore_empty);

// Insert/delete primitives used by many editing operations
void ins_bytes(char_u *p);
void ins_bytes_len(char_u *p, int len);
void ins_char(int c);
void ins_char_bytes(char_u *buf, int charlen);
void ins_str(char_u *s, size_t slen);
int  del_char(int fixpos);
int  del_chars(long count, int fixpos);
int  del_bytes(long count, int fixpos_arg, int use_delcombine);
int  open_line(int dir, int flags, int second_line_indent, int *did_do_comment);
int  truncate_line(int fixpos);
void del_lines(long nlines, int undo);
