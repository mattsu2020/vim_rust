#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Equivalent of Vim's change_warning().  Returns 1 when a warning should be
 * displayed and 0 otherwise.  Only warns once while the buffer is marked as
 * readonly and not yet changed.
 */
int change_warning(int _col);

/**
 * Mark the buffer as changed.  This mirrors Vim's changed() which notifies
 * the editor that modifications have been made.
 */
void changed(void);

void changed_internal(void);

void f_listener_add(void *_argvars, void *_rettv);

void f_listener_flush(void *_argvars, void *_rettv);

void f_listener_remove(void *_argvars, void *_rettv);

void may_invoke_listeners(void *_buf, long _lnum, long _lnume, int _added);

void invoke_listeners(void *_buf);

void remove_listeners(void *_buf);

void changed_bytes(long _lnum, int _col);

void inserted_bytes(long _lnum, int _col, int _added);

void appended_lines(long _lnum, long _count);

void appended_lines_mark(long _lnum, long _count);

void deleted_lines(long _lnum, long _count);

void deleted_lines_mark(long _lnum, long _count);

void changed_lines_buf(void *_buf, long _lnum, long _lnume, long _xtra);

void changed_lines(long _lnum, int _col, long _lnume, long _xtra);

void unchanged(void *_buf, int _ff, int _always_inc_changedtick);

void save_file_ff(void *_buf);

int file_ff_differs(void *_buf, int _ignore_empty);

void ins_bytes(uint8_t *_p);

void ins_bytes_len(uint8_t *_p, int _len);

void ins_char(int _c);

void ins_char_bytes(uint8_t *_buf, int _charlen);

void ins_str(uint8_t *_s, uintptr_t _slen);

int del_char(int _fixpos);

int del_chars(long _count, int _fixpos);

int del_bytes(long _count, int _fixpos_arg, int _use_delcombine);

int open_line(int _dir, int _flags, int _second_line_indent, int *did_do_comment);

int truncate_line(int _fixpos);

void del_lines(long _nlines, int _undo);
